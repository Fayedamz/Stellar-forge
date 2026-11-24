// Copyright 2024 Stellar-K8s Contributors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Decentralized Oracles Aggregator with Medianizer & Outlier Trimming
//!
//! A manipulation-resistant on-chain price aggregator for Stellar Soroban that:
//!
//! * Accepts price pushes from up to [`math::MAX_FEEDS`] authorized oracle feeds.
//! * Computes a **median price** over the submitted values.
//! * Removes statistical outliers via configurable percentage-deviation trimming
//!   ([`math::trim_outliers`]).
//! * Enforces **heartbeat staleness checks** — prices older than
//!   `heartbeat_ledgers` are rejected and excluded from aggregation.
//! * Exposes a `get_price` view that always returns the latest aggregated result.
//!
//! ## Design
//!
//! Authorized feeds call [`OracleAggregator::submit_price`] once per heartbeat
//! window.  The aggregator stores each submission in Instance Storage keyed by
//! feed address, then re-computes the aggregate lazily when `get_price` is
//! called (or eagerly after each submission via `aggregate`).
//!
//! All arithmetic is performed with `i128` fixed-point integers scaled by
//! [`math::PRICE_PRECISION`] (10^7) — no floating-point, no overflow risk.
//!
//! ## Storage keys
//!
//! | Key variant | Value | Lifetime |
//! |-------------|-------|----------|
//! | `Config`    | [`AggregatorConfig`] | Instance (persistent) |
//! | `Feed(addr)`| [`FeedEntry`] | Instance (persistent) |
//! | `AggPrice`  | [`AggregatedPrice`] | Instance (persistent) |

#![no_std]

pub mod math;

use math::{median, trim_outliers, MAX_FEEDS, MAX_FEEDS_U32};

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, Vec,
};

// ---------------------------------------------------------------------------
// Error codes
// ---------------------------------------------------------------------------

/// Contract-level error codes returned to callers.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AggregatorError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// The caller is not the admin.
    Unauthorized = 3,
    /// The feed address is already registered.
    FeedAlreadyRegistered = 4,
    /// The feed address is not registered.
    FeedNotFound = 5,
    /// The maximum number of feeds has been reached.
    MaxFeedsReached = 6,
    /// The submitted price is non-positive or overflows.
    InvalidPrice = 7,
    /// The deviation threshold is out of range (0 – 10_000 bps).
    InvalidDeviation = 8,
    /// The heartbeat window must be at least 1 ledger.
    InvalidHeartbeat = 9,
    /// Not enough valid (non-stale) feeds to compute an aggregate.
    InsufficientFeeds = 10,
    /// The submitted price is stale (older than heartbeat window).
    StaleFeed = 11,
}

// ---------------------------------------------------------------------------
// Contract types
// ---------------------------------------------------------------------------

/// Persisted configuration set during [`OracleAggregator::initialize`].
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregatorConfig {
    /// Contract administrator — allowed to register/remove feeds and update config.
    pub admin: Address,
    /// Outlier-trimming threshold in basis-points (1 bps = 0.01%).
    /// E.g. `2000` = 20%.  Prices deviating more than this fraction from the
    /// preliminary median are excluded from the final aggregate.
    pub deviation_bps: u32,
    /// Staleness window in ledgers.  A feed submission is considered stale if
    /// `current_ledger - submission_ledger > heartbeat_ledgers`.
    pub heartbeat_ledgers: u32,
    /// Minimum number of valid (non-stale, non-outlier) feeds required to
    /// produce an aggregate.
    pub min_feeds: u32,
}

/// A single price submission from an authorized oracle feed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeedEntry {
    /// Feed oracle address (used for authorization).
    pub feed: Address,
    /// Most recently submitted price, scaled by [`PRICE_PRECISION`].
    pub price: i128,
    /// Ledger sequence number at the time of the last submission.
    pub last_ledger: u32,
}

/// Aggregated price result stored after each successful aggregation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregatedPrice {
    /// The aggregated median price, scaled by [`PRICE_PRECISION`].
    pub price: i128,
    /// Ledger at which this aggregate was computed.
    pub ledger: u32,
    /// Number of feeds that contributed to this aggregate (after outlier trim).
    pub contributing_feeds: u32,
}

/// Storage key variants.
#[contracttype]
enum DataKey {
    /// Singleton configuration.
    Config,
    /// Registration flag per feed address; value = true when registered.
    Registered(Address),
    /// Per-feed price submission; keyed by feed `Address`.
    Feed(Address),
    /// Latest aggregated price.
    AggPrice,
    /// Ordered list of registered feed addresses (for enumeration).
    FeedList,
}

// ---------------------------------------------------------------------------
// Contract definition
// ---------------------------------------------------------------------------

#[contract]
pub struct OracleAggregator;

#[contractimpl]
impl OracleAggregator {
    // -----------------------------------------------------------------------
    // Lifecycle
    // -----------------------------------------------------------------------

    /// Initialize the oracle aggregator.
    ///
    /// Must be called exactly once.
    ///
    /// # Parameters
    ///
    /// - `admin`: Address that may manage feeds and config.
    /// - `deviation_bps`: Outlier threshold in basis-points (0 – 10_000).
    /// - `heartbeat_ledgers`: How many ledgers a price remains fresh (≥ 1).
    /// - `min_feeds`: Minimum valid feeds required to produce a price (≥ 1).
    pub fn initialize(
        env: Env,
        admin: Address,
        deviation_bps: u32,
        heartbeat_ledgers: u32,
        min_feeds: u32,
    ) -> Result<(), AggregatorError> {
        if env.storage().instance().has(&DataKey::Config) {
            panic_with_error!(&env, AggregatorError::AlreadyInitialized);
        }

        if deviation_bps > 10_000 {
            panic_with_error!(&env, AggregatorError::InvalidDeviation);
        }
        if heartbeat_ledgers == 0 {
            panic_with_error!(&env, AggregatorError::InvalidHeartbeat);
        }
        if min_feeds == 0 || min_feeds > MAX_FEEDS_U32 {
            panic_with_error!(&env, AggregatorError::InsufficientFeeds);
        }

        admin.require_auth();

        let config = AggregatorConfig {
            admin,
            deviation_bps,
            heartbeat_ledgers,
            min_feeds,
        };
        env.storage().instance().set(&DataKey::Config, &config);

        // Initialize empty feed list
        let feed_list: Vec<Address> = Vec::new(&env);
        env.storage().instance().set(&DataKey::FeedList, &feed_list);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Feed management (admin-only)
    // -----------------------------------------------------------------------

    /// Register a new authorized oracle feed.
    ///
    /// Requires admin authorization.  Up to [`MAX_FEEDS_U32`] feeds may be
    /// registered.
    pub fn add_feed(env: Env, feed: Address) -> Result<(), AggregatorError> {
        let config = Self::load_config(&env)?;
        config.admin.require_auth();

        // Check capacity
        let mut feed_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::FeedList)
            .unwrap_or_else(|| Vec::new(&env));

        if feed_list.len() >= MAX_FEEDS_U32 {
            panic_with_error!(&env, AggregatorError::MaxFeedsReached);
        }

        // Check for duplicate
        if env
            .storage()
            .instance()
            .has(&DataKey::Registered(feed.clone()))
        {
            panic_with_error!(&env, AggregatorError::FeedAlreadyRegistered);
        }

        feed_list.push_back(feed.clone());
        env.storage().instance().set(&DataKey::FeedList, &feed_list);
        env.storage()
            .instance()
            .set(&DataKey::Registered(feed), &true);

        Ok(())
    }

    /// Remove an authorized oracle feed.
    ///
    /// Requires admin authorization.  Any stored price from this feed is also
    /// removed.
    pub fn remove_feed(env: Env, feed: Address) -> Result<(), AggregatorError> {
        let config = Self::load_config(&env)?;
        config.admin.require_auth();

        if !env
            .storage()
            .instance()
            .has(&DataKey::Registered(feed.clone()))
        {
            panic_with_error!(&env, AggregatorError::FeedNotFound);
        }

        env.storage()
            .instance()
            .remove(&DataKey::Registered(feed.clone()));
        // Also remove any price submission if present
        if env.storage().instance().has(&DataKey::Feed(feed.clone())) {
            env.storage()
                .instance()
                .remove(&DataKey::Feed(feed.clone()));
        }

        // Remove from feed list
        let feed_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::FeedList)
            .unwrap_or_else(|| Vec::new(&env));

        let mut new_list: Vec<Address> = Vec::new(&env);
        for i in 0..feed_list.len() {
            let addr = feed_list.get(i).unwrap();
            if addr != feed {
                new_list.push_back(addr);
            }
        }
        env.storage().instance().set(&DataKey::FeedList, &new_list);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Price submission
    // -----------------------------------------------------------------------

    /// Submit a price from an authorized oracle feed.
    ///
    /// - `feed`: Must be a registered feed address and must authenticate.
    /// - `price`: Scaled by [`PRICE_PRECISION`].  Must be positive.
    ///
    /// On success the aggregator re-computes the aggregate price if the
    /// resulting set of valid feeds meets `min_feeds`.
    pub fn submit_price(env: Env, feed: Address, price: i128) -> Result<(), AggregatorError> {
        Self::load_config(&env)?; // ensure initialized

        // Verify the feed is registered
        if !env
            .storage()
            .instance()
            .has(&DataKey::Registered(feed.clone()))
        {
            panic_with_error!(&env, AggregatorError::FeedNotFound);
        }

        // The feed must authorize this call
        feed.require_auth();

        if price <= 0 {
            panic_with_error!(&env, AggregatorError::InvalidPrice);
        }

        let entry = FeedEntry {
            feed: feed.clone(),
            price,
            last_ledger: env.ledger().sequence(),
        };
        env.storage().instance().set(&DataKey::Feed(feed), &entry);

        // Eagerly update the aggregate (best-effort; errors are non-fatal)
        let _ = Self::aggregate_internal(&env);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Aggregation
    // ------------------------------------------------------
