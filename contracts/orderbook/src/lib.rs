#![no_std]

pub mod queue;

use queue::PriceLevelQueue;
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Vec};

/// Minimum order price, in base units of the quote asset.
const MIN_PRICE: u32 = 1;
/// Maximum order price, in base units of the quote asset.
const MAX_PRICE: u32 = 10_000;

const INSTANCE_TTL_THRESHOLD: u32 = 100;
const INSTANCE_TTL_BUMP: u32 = 5000;

#[contracterror]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidPrice = 4,
    InvalidAmount = 5,
    OrderNotFound = 6,
    InvalidSide = 7,
    OrderAlreadyFilled = 8,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Order {
    pub id: u64,
    pub trader: Address,
    pub side: Side,
    pub price: u32,
    pub amount: i128,
    pub filled: i128,
    pub created_at: u32,
}

impl Order {
    /// Remaining amount that can still be filled.
    pub fn remaining(&self) -> i128 {
        self.amount - self.filled
    }

    /// Returns `true` when the order is fully filled.
    pub fn is_filled(&self) -> bool {
        self.filled >= self.amount
    }
}

/// An aggregated price level holding the FIFO queue of its order identifiers.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceLevel {
    pub price: u32,
    pub queue: PriceLevelQueue,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    NextOrderId,
    BuyLevels,
    SellLevels,
    Order(u64),
}

#[contract]
pub struct Orderbook;

#[contractimpl]
impl Orderbook {
    /// Initializes the orderbook with an administrator.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextOrderId, &1u64);
        env.storage()
            .instance()
            .set(&DataKey::BuyLevels, &empty_levels(&env));
        env.storage()
            .instance()
            .set(&DataKey::SellLevels, &empty_levels(&env));
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_BUMP);
        Ok(())
    }

    /// Places a limit order and returns the new order identifier.
    pub fn place_order(
        env: Env,
        trader: Address,
        side: Side,
        price: u32,
        amount: i128,
    ) -> Result<u64, Error> {
        trader.require_auth();
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }
        if !(MIN_PRICE..=MAX_PRICE).contains(&price) {
            return Err(Error::InvalidPrice);
        }
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let next_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextOrderId)
            .ok_or(Error::NotInitialized)?;
        env.storage()
            .instance()
            .set(&DataKey::NextOrderId, &(next_id + 1));

        let order = Order {
            id: next_id,
            trader: trader.clone(),
            side,
            price,
            amount,
            filled: 0,
            created_at: env.ledger().sequence(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Order(next_id), &order);

        let mut levels = Self::levels(&env, side);
        enqueue_order(&env, &mut levels, price, next_id);
        env.storage().instance().set(&side_key(side), &levels);
        Ok(next_id)
    }

    /// Cancels an open order. Only the order owner may cancel it.
    pub fn cancel_order(env: Env, order_id: u64) -> Result<(), Error> {
        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::NotInitialized);
        }
        let mut order: Order = env
            .storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .ok_or(Error::OrderNotFound)?;
        order.trader.require_auth();

        if order.is_filled() {
            return Err(Error::OrderAlreadyFilled);
        }

        let mut levels = Self::levels(&env, order.side);
        for i in 0..levels.len() {
            let level = levels.get(i).unwrap();
            if level.price != order.price {
                continue;
            }
            let mut level = level;
            let removed = level.queue.remove(order_id);
            if removed && level.queue.is_empty() {
                levels.remove(i);
            } else {
                levels.set(i, level);
            }
            break;
        }
        env.storage().instance().set(&side_key(order.side), &levels);

        order.filled = order.amount;
        env.storage()
            .persistent()
            .set(&DataKey::Order(order_id), &order);
        Ok(())
    }

    /// Returns an order by identifier.
    pub fn get_order(env: Env, order_id: u64) -> Result<Order, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Order(order_id))
            .ok_or(Error::OrderNotFound)
    }

    /// Returns the order identifiers queued at a price level, oldest first.
    pub fn get_level(env: Env, side: Side, price: u32) -> Vec<u64> {
        let levels = Self::levels(&env, side);
        for level in levels.iter() {
            if level.price == price {
                return level.queue.order_ids(&env);
            }
        }
        Vec::new(&env)
    }

    /// Returns the best price with open orders: the highest buy or the
    /// lowest sell. Levels are stored in descending price order, so the
    /// best buy is the first level and the best sell is the last.
    pub fn best_price(env: Env, side: Side) -> Option<u32> {
        let levels = Self::levels(&env, side);
        if levels.is_empty() {
            None
        } else {
            match side {
                Side::Buy => levels.first().map(|l| l.price),
                Side::Sell => levels.last().map(|l| l.price),
            }
        }
    }

    /// Returns the number of distinct price levels with open orders.
    pub fn level_count(env: Env, side: Side) -> u32 {
        Self::levels(&env, side).len()
    }

    fn levels(env: &Env, side: Side) -> Vec<PriceLevel> {
        env.storage()
            .instance()
            .get(&side_key(side))
            .unwrap_or_else(|| empty_levels(env))
    }
}

fn side_key(side: Side) -> DataKey {
    match side {
        Side::Buy => DataKey::BuyLevels,
        Side::Sell => DataKey::SellLevels,
    }
}

fn empty_levels(env: &Env) -> Vec<PriceLevel> {
    Vec::new(env)
}

/// Inserts an order identifier into the level for `price`,
/// keeping levels sorted by descending price.
fn enqueue_order(env: &Env, levels: &mut Vec<PriceLevel>, price: u32, order_id: u64) {
    for i in 0..levels.len() {
        let level = levels.get(i).unwrap();
        if level.price == price {
            let mut level = level;
            level.queue.push(order_id);
            levels.set(i, level);
            return;
        }
    }

    let queue = {
        let mut q = PriceLevelQueue::new(env);
        q.push(order_id);
        q
    };
    let new_level = PriceLevel { price, queue };

    let mut insert_at = levels.len();
    for i in 0..levels.len() {
        if levels.get(i).unwrap().price < price {
            insert_at = i;
            break;
        }
    }
    levels.insert(insert_at, new_level);
}

#[cfg(test)]
mod tests {
    use super::{Error, Orderbook, OrderbookClient, Side};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    fn setup() -> (Env, OrderbookClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register(Orderbook, ());
        let client = OrderbookClient::new(&env, &contract_id);
        client.initialize(&admin);
        (env, client, admin)
    }

    #[test]
    fn initialize_is_guarded() {
        let (env, client, admin) = setup();
        let res = client.try_initialize(&admin);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
        assert_ne!(admin, Address::generate(&env));
    }

    #[test]
    fn place_and_query_orders() {
        let (_env, client, trader) = setup();
        let id1 = client.place_order(&trader, &Side::Buy, &100, &1_000);
        let id2 = client.place_order(&trader, &Side::Buy, &100, &500);
        let id3 = client.place_order(&trader, &Side::Sell, &110, &700);

        assert_eq!(client.best_price(&Side::Buy), Some(100));
        assert_eq!(client.best_price(&Side::Sell), Some(110));

        let level = client.get_level(&Side::Buy, &100);
        assert_eq!(level.get(0), Some(id1));
        assert_eq!(level.get(1), Some(id2));

        let order = client.get_order(&id3);
        assert_eq!(order.side, Side::Sell);
        assert_eq!(order.remaining(), 700);
    }

    #[test]
    fn cancel_removes_from_queue() {
        let (_env, client, trader) = setup();
        let id1 = client.place_order(&trader, &Side::Buy, &100, &1_000);
        let id2 = client.place_order(&trader, &Side::Buy, &100, &500);

        client.cancel_order(&id1);
        let level = client.get_level(&Side::Buy, &100);
        assert_eq!(level.get(0), Some(id2));

        let order = client.get_order(&id1);
        assert!(order.is_filled());
        let res = client.try_cancel_order(&id1);
        assert_eq!(res, Err(Ok(Error::OrderAlreadyFilled)));
    }

    #[test]
    fn rejects_invalid_input() {
        let (_env, client, trader) = setup();
        let res = client.try_place_order(&trader, &Side::Buy, &0, &100);
        assert_eq!(res, Err(Ok(Error::InvalidPrice)));
        let res = client.try_place_order(&trader, &Side::Buy, &100, &0);
        assert_eq!(res, Err(Ok(Error::InvalidAmount)));
        let res = client.try_get_order(&999);
        assert_eq!(res, Err(Ok(Error::OrderNotFound)));
    }

    #[test]
    fn levels_stay_sorted() {
        let (_env, client, trader) = setup();
        client.place_order(&trader, &Side::Sell, &110, &100);
        client.place_order(&trader, &Side::Sell, &90, &100);
        client.place_order(&trader, &Side::Sell, &100, &100);

        let levels = client.level_count(&Side::Sell);
        assert_eq!(levels, 3);

        let lowest = client.best_price(&Side::Sell);
        assert_eq!(lowest, Some(90));

        let level = client.get_level(&Side::Sell, &90);
        assert_eq!(level.len(), 1);
    }
}
