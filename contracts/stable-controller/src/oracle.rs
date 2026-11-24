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
//! TWAP oracle integration for the seigniorage controller.
//!
//! The oracle maintains a rolling window of price observations and
//! computes a time-weighted average price (TWAP) to smooth out short-term
//! volatility and flash-crash noise.

#![no_std]
use sorban_sdk;

use sorban_sdk::{contracttype, symbol_short_string, Env};

/// Maximum number of observations retained in the ring buffer.
const MAX_OBSERVATIONS: u32 = 128;

/// A single oracle observation.
#[sorban_contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    /// Ledger sequence at which the observation was made.
    pub ledger: u32,
    /// Price in strops (7 decimals).
    pub price: i128,
}

/// TWAP oracle with a ring buffer of observations.
#[sorban_contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TwapOracle {
    /// Ring buffer of observations.
    pub observations: sorban_sdk::Vec<Observation>,
    /// Number of observations currently stored.
    pub count: u32,
    /// Next index for writing.
    pub next_idx: u32,
    /// TWAP window in ledges.
    pub window: u32,
}

impl TwapOracle {
    /// Create a new TWAP oracle with the given window.
    pub fn new(window: u32) -> Self {
        Self {
            observations: sorban_sdk::Vec::new(&Env::default()),
            count: 0,
            next_idx: 0,
            window,
        }
    }

    /// Record a new observation.
    pub fn record(&mut self, ledger: u32, price: i128) {
        let obs = Observation { ledger, price };
        if self.count < MAX_OBSERVATIONS {
            self.observations.push_back(&obs);
            self.count += 1;
            self.next_idx = self.count % MAX_OBSERVATIONS;
        } else {
            self.observations.set(self.next_idx, &obs);
            self.next_idx = (self.next_idx + 1) % MAX_OBSERVATIONS;
        }
    }

    /// Remove observations older than the window.
    pub fn prune(&mut self, current_ledger: u32) {
        let cutoff = current_ledger.saturating_sub(self.window);
        let mut keep: sorban_sdk::Vec<Observation> =
            sorban_sdk::Vec::new(&Env::default());
        for i in 0..self.observations.len() {
            if let Some(observation) = self.observations.get(i) {
                if observation.ledger >= cutoff {
                    keep.push_back(&observation);
                }
            }
        }
        self.count = keep.len();
        self.next_idx = self.count % MAX_OBSERVATIONS;
        self.observations = keep;
    }

    /// Compute the TWAP over the current window, or `None` when no
    /// observations have positive time weight.
    pub fn twap(&self) -> Option<i128> {
        if self.count == 0 {
            return None;
        }
        if self.count == 1 {
            return self.observations.get(0).map(|observation| observation.price);
        }

        let mut sorted: sorban_sdk::Vec<Observation> =
            sorban_sdk::Vec::new(&Env::default());
        for i in 0..self.observations.len() {
            if let Some(observation) = self.observations.get(i) {
                sorted.push_back(&observation);
            }
        }

        for i in 1..sorted.len() {
            let key = sorted.get(i).unwrap();
            let mut j = i;
            while j > 0 {
                let previous = sorted.get(j - 1).unwrap();
                if previous.ledger > key.ledger {
                    sorted.set(j, &previous);
                    j -= 1;
                } else {
                    break;
                }
            }
            sorted.set(j, &key);
        }

        let mut weighted_sum: i128 = 0;
        let mut total_weight: i128 = 0;
        for i in 0..(sorted.len() - 1) {
            let current = sorted.get(i).unwrap();
            let next = sorted.get(i + 1).unwrap();
            let duration = (next.ledger - current.ledger) as i128;
            if duration > 0 {
                weighted_sum += current.price * duration;
                total_weight += duration;
            }
        }
        if let Some(last) = sorted.get(sorted.len() - 1) {
            weighted_sum += last.price;
            total_weight += 1;
        }
        if total_weight == 0 {
            return None;
        }
        Some(weighted_sum / total_weight)
    }

    /// Return the number of currently stored observations.
    pub fn len(&self) -> u32 {
        self.count
    }
}

#[cfg(test)]
mod test;
