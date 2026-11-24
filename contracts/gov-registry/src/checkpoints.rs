use soroban_sdk::{contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BalanceCheckpoint {
    pub ledger: u32,
    pub balance: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationCheckpoint {
    pub ledger: u32,
    pub delegatee: Option<Address>,
}

pub fn balance_at(checkpoints: &Vec<BalanceCheckpoint>, ledger: u32) -> i128 {
    let mut result = 0;
    for checkpoint in checkpoints.iter() {
        if checkpoint.ledger > ledger {
            break;
        }
        result = checkpoint.balance;
    }
    result
}

pub fn delegation_at(checkpoints: &Vec<DelegationCheckpoint>, ledger: u32) -> Option<Address> {
    let mut result = None;
    for checkpoint in checkpoints.iter() {
        if checkpoint.ledger > ledger {
            break;
        }
        result = checkpoint.delegatee;
    }
    result
}

pub fn write_balance(env: &Env, key: &(soroban_sdk::Symbol, Address), value: i128) {
    let ledger = env.ledger().sequence();
    let mut checkpoints: Vec<BalanceCheckpoint> = env
        .storage()
        .persistent()
        .get(key)
        .unwrap_or_else(|| Vec::new(env));
    let checkpoint = BalanceCheckpoint {
        ledger,
        balance: value,
    };
    if let Some(last) = checkpoints.pop_back() {
        if last.ledger == ledger {
            checkpoints.push_back(checkpoint);
        } else {
            checkpoints.push_back(last);
            checkpoints.push_back(checkpoint);
        }
    } else {
        checkpoints.push_back(checkpoint);
    }
    env.storage().persistent().set(key, &checkpoints);
}

pub fn write_delegation(
    env: &Env,
    key: &(soroban_sdk::Symbol, Address),
    delegatee: Option<Address>,
) {
    let ledger = env.ledger().sequence();
    let mut checkpoints: Vec<DelegationCheckpoint> = env
        .storage()
        .persistent()
        .get(key)
        .unwrap_or_else(|| Vec::new(env));
    let checkpoint = DelegationCheckpoint { ledger, delegatee };
    if let Some(last) = checkpoints.pop_back() {
        if last.ledger == ledger {
            checkpoints.push_back(checkpoint);
        } else {
            checkpoints.push_back(last);
            checkpoints.push_back(checkpoint);
        }
    } else {
        checkpoints.push_back(checkpoint);
    }
    env.storage().persistent().set(key, &checkpoints);
}
