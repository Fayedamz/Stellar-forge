//! FIFO queue of order identifiers for a single price level.

use soroban_sdk::{contracttype, Env, Vec};

/// A FIFO queue of order identifiers stored as a Soroban vector.
///
/// Orders are appended at the back and filled from the front, which gives
/// time priority to the order that was placed first at a given price.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceLevelQueue {
    order_ids: Vec<u64>,
}

impl PriceLevelQueue {
    /// Creates an empty queue.
    pub fn new(env: &Env) -> Self {
        Self {
            order_ids: Vec::new(env),
        }
    }

    /// Creates a queue from an existing vector of order identifiers.
    pub fn from_vec(env: &Env, order_ids: Vec<u64>) -> Self {
        let mut out = Vec::new(env);
        for id in order_ids.iter() {
            out.push_back(id);
        }
        Self { order_ids: out }
    }

    /// Appends an order identifier to the back of the queue.
    pub fn push(&mut self, order_id: u64) {
        self.order_ids.push_back(order_id);
    }

    /// Removes and returns the front order identifier.
    pub fn pop_front(&mut self) -> Option<u64> {
        let front = self.peek()?;
        self.order_ids.remove(0);
        Some(front)
    }

    /// Returns the front order identifier without removing it.
    pub fn peek(&self) -> Option<u64> {
        if self.order_ids.is_empty() {
            None
        } else {
            self.order_ids.get(0)
        }
    }

    /// Removes a specific order identifier from the queue.
    ///
    /// Returns `true` when the identifier was present.
    pub fn remove(&mut self, order_id: u64) -> bool {
        for i in 0..self.order_ids.len() {
            if self.order_ids.get(i) == Some(order_id) {
                self.order_ids.remove(i);
                return true;
            }
        }
        false
    }

    /// Returns `true` when the queue holds the given order identifier.
    pub fn contains(&self, order_id: u64) -> bool {
        for i in 0..self.order_ids.len() {
            if self.order_ids.get(i) == Some(order_id) {
                return true;
            }
        }
        false
    }

    /// Number of order identifiers currently queued.
    pub fn len(&self) -> u32 {
        self.order_ids.len()
    }

    /// Returns `true` when the queue holds no order identifiers.
    pub fn is_empty(&self) -> bool {
        self.order_ids.is_empty()
    }

    /// Returns a copy of the queued order identifiers, front first.
    pub fn order_ids(&self, env: &Env) -> Vec<u64> {
        let mut out = Vec::new(env);
        for id in self.order_ids.iter() {
            out.push_back(id);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::PriceLevelQueue;
    use soroban_sdk::Env;

    #[test]
    fn push_pop_fifo_order() {
        let env = Env::default();
        let mut q = PriceLevelQueue::new(&env);
        assert!(q.is_empty());

        q.push(1);
        q.push(2);
        q.push(3);
        assert_eq!(q.len(), 3);

        assert_eq!(q.pop_front(), Some(1));
        assert_eq!(q.pop_front(), Some(2));
        assert_eq!(q.pop_front(), Some(3));
        assert_eq!(q.pop_front(), None);
        assert!(q.is_empty());
    }

    #[test]
    fn peek_does_not_remove() {
        let env = Env::default();
        let mut q = PriceLevelQueue::new(&env);
        assert_eq!(q.peek(), None);
        q.push(7);
        assert_eq!(q.peek(), Some(7));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn remove_specific_order() {
        let env = Env::default();
        let mut q = PriceLevelQueue::from_vec(&env, {
            let mut v = soroban_sdk::Vec::new(&env);
            v.push_back(1);
            v.push_back(2);
            v.push_back(3);
            v
        });
        assert!(q.contains(2));
        assert!(q.remove(2));
        assert!(!q.contains(2));
        assert!(!q.remove(2));
        assert_eq!(q.order_ids(&env), {
            let mut v = soroban_sdk::Vec::new(&env);
            v.push_back(1);
            v.push_back(3);
            v
        });
    }
}
