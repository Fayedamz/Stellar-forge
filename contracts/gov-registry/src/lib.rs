#![no_std]

pub mod checkpoints;

use checkpoints::{
    balance_at, delegation_at, write_balance, write_delegation, BalanceCheckpoint,
    DelegationCheckpoint,
};
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, Address, Env,
    Symbol, Vec,
};

const ADMIN: Symbol = symbol_short!("ADMIN");
const ACCOUNTS: Symbol = symbol_short!("ACCTS");
const BALANCE: Symbol = symbol_short!("BAL");
const DELEGATION: Symbol = symbol_short!("DELEG");
const PROPOSAL: Symbol = symbol_short!("PROP");
const VOTED: Symbol = symbol_short!("VOTED");
const SNAPSHOT: Symbol = symbol_short!("SNAP");

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Vote {
    For,
    Against,
    Abstain,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProposalState {
    Active,
    Passed,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub title: Symbol,
    pub snapshot_ledger: u32,
    pub deadline: u32,
    pub quorum_bps: u32,
    pub total_power: i128,
    pub for_weight: i128,
    pub against_weight: i128,
    pub abstain_weight: i128,
    pub state: ProposalState,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Unauthorized = 1,
    InvalidAmount = 2,
    InsufficientBalance = 3,
    InvalidQuorum = 4,
    InvalidDuration = 5,
    InvalidThreshold = 6,
    DelegationCycle = 7,
    ProposalNotFound = 8,
    VotingClosed = 9,
    VotingStillOpen = 10,
    AlreadyVoted = 11,
    NoVotingPower = 12,
    AlreadyFinalized = 13,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalCreated {
    #[topic]
    pub id: u32,
    pub snapshot_ledger: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteCast {
    #[topic]
    pub id: u32,
    pub voter: Address,
    pub vote: Vote,
    pub weight: i128,
}

#[contract]
pub struct GovRegistry;

#[contractimpl]
impl GovRegistry {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&ADMIN, &admin);
        env.storage()
            .instance()
            .set(&ACCOUNTS, &Vec::<Address>::new(&env));
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        Self::require_admin(&env);
        if amount <= 0 {
            env.panic_with_error(&Error::InvalidAmount);
        }
        register(&env, &to);
        let key = (BALANCE, to.clone());
        write_balance(&env, &key, balance(&env, &to) + amount);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        if amount <= 0 {
            env.panic_with_error(&Error::InvalidAmount);
        }
        let current = balance(&env, &from);
        if current < amount {
            env.panic_with_error(&Error::InsufficientBalance);
        }
        register(&env, &from);
        register(&env, &to);
        write_balance(&env, &(BALANCE, from), current - amount);
        write_balance(&env, &(BALANCE, to.clone()), balance(&env, &to) + amount);
    }

    pub fn delegate(env: Env, delegator: Address, delegatee: Option<Address>) {
        delegator.require_auth();
        if let Some(target) = delegatee.clone() {
            register(&env, &target);
            let mut cursor = Some(target);
            let accounts = accounts(&env);
            for _ in 0..=accounts.len() {
                if cursor == Some(delegator.clone()) {
                    env.panic_with_error(&Error::DelegationCycle);
                }
                cursor =
                    cursor.and_then(|address| delegation(&env, &address, env.ledger().sequence()));
                if cursor.is_none() {
                    break;
                }
            }
        }
        register(&env, &delegator);
        write_delegation(&env, &(DELEGATION, delegator), delegatee);
    }

    pub fn voting_power(env: Env, account: Address, ledger: u32) -> i128 {
        voting_power_at(&env, &account, ledger)
    }

    pub fn propose(
        env: Env,
        proposer: Address,
        title: Symbol,
        voting_duration: u32,
        quorum_bps: u32,
        minimum_power: i128,
    ) -> u32 {
        proposer.require_auth();
        if voting_duration == 0 {
            env.panic_with_error(&Error::InvalidDuration);
        }
        if quorum_bps == 0 || quorum_bps > 10_000 {
            env.panic_with_error(&Error::InvalidQuorum);
        }
        if minimum_power < 0 {
            env.panic_with_error(&Error::InvalidThreshold);
        }
        let snapshot = env.ledger().sequence();
        if voting_power_at(&env, &proposer, snapshot) < minimum_power {
            env.panic_with_error(&Error::InvalidThreshold);
        }
        let id = env
            .storage()
            .instance()
            .get::<_, u32>(&PROPOSAL)
            .unwrap_or(0)
            + 1;
        env.storage().instance().set(&PROPOSAL, &id);
        let proposal = Proposal {
            id,
            proposer,
            title,
            snapshot_ledger: snapshot,
            deadline: snapshot.saturating_add(voting_duration),
            quorum_bps,
            total_power: total_voting_power(&env, snapshot),
            for_weight: 0,
            against_weight: 0,
            abstain_weight: 0,
            state: ProposalState::Active,
        };
        for account in accounts(&env).iter() {
            let weight = voting_power_at(&env, &account, snapshot);
            env.storage()
                .persistent()
                .set(&(SNAPSHOT, id, account), &weight);
        }
        env.storage().persistent().set(&(PROPOSAL, id), &proposal);
        ProposalCreated {
            id,
            snapshot_ledger: snapshot,
        }
        .publish(&env);
        id
    }

    pub fn vote(env: Env, voter: Address, proposal_id: u32, vote: Vote) {
        voter.require_auth();
        let key = (PROPOSAL, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| env.panic_with_error(&Error::ProposalNotFound));
        if env.ledger().sequence() >= proposal.deadline {
            env.panic_with_error(&Error::VotingClosed);
        }
        if env
            .storage()
            .persistent()
            .has(&(VOTED, proposal_id, voter.clone()))
        {
            env.panic_with_error(&Error::AlreadyVoted);
        }
        let weight: i128 = env
            .storage()
            .persistent()
            .get(&(SNAPSHOT, proposal_id, voter.clone()))
            .unwrap_or(0);
        if weight <= 0 {
            env.panic_with_error(&Error::NoVotingPower);
        }
        match vote {
            Vote::For => proposal.for_weight += weight,
            Vote::Against => proposal.against_weight += weight,
            Vote::Abstain => proposal.abstain_weight += weight,
        }
        env.storage().persistent().set(&key, &proposal);
        env.storage()
            .persistent()
            .set(&(VOTED, proposal_id, voter.clone()), &true);
        VoteCast {
            id: proposal_id,
            voter,
            vote,
            weight,
        }
        .publish(&env);
    }

    pub fn finalize(env: Env, proposal_id: u32) {
        let key = (PROPOSAL, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| env.panic_with_error(&Error::ProposalNotFound));
        if proposal.state != ProposalState::Active {
            env.panic_with_error(&Error::AlreadyFinalized);
        }
        if env.ledger().sequence() < proposal.deadline {
            env.panic_with_error(&Error::VotingStillOpen);
        }
        let total = proposal.for_weight + proposal.against_weight + proposal.abstain_weight;
        let quorum = (proposal.total_power * proposal.quorum_bps as i128 + 9_999) / 10_000;
        proposal.state = if total >= quorum && proposal.for_weight > proposal.against_weight {
            ProposalState::Passed
        } else {
            ProposalState::Rejected
        };
        env.storage().persistent().set(&key, &proposal);
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Option<Proposal> {
        env.storage().persistent().get(&(PROPOSAL, proposal_id))
    }
}

impl GovRegistry {
    fn require_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
    }
}

fn register(env: &Env, account: &Address) {
    let mut list = accounts(env);
    if !list.iter().any(|candidate| candidate == *account) {
        list.push_back(account.clone());
        env.storage().instance().set(&ACCOUNTS, &list);
    }
}

fn accounts(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&ACCOUNTS)
        .unwrap_or_else(|| Vec::new(env))
}

fn balance(env: &Env, account: &Address) -> i128 {
    let checkpoints: Vec<BalanceCheckpoint> = env
        .storage()
        .persistent()
        .get(&(BALANCE, account.clone()))
        .unwrap_or_else(|| Vec::new(env));
    balance_at(&checkpoints, env.ledger().sequence())
}

fn delegation(env: &Env, account: &Address, ledger: u32) -> Option<Address> {
    let checkpoints: Vec<DelegationCheckpoint> = env
        .storage()
        .persistent()
        .get(&(DELEGATION, account.clone()))
        .unwrap_or_else(|| Vec::new(env));
    delegation_at(&checkpoints, ledger)
}

fn terminal_delegate(env: &Env, account: &Address, ledger: u32) -> Address {
    let mut current = account.clone();
    let mut visited = Vec::new(env);
    for _ in 0..=accounts(env).len() {
        if visited.iter().any(|candidate| candidate == current) {
            break;
        }
        visited.push_back(current.clone());
        match delegation(env, &current, ledger) {
            Some(next) => current = next,
            None => break,
        }
    }
    current
}

fn voting_power_at(env: &Env, target: &Address, ledger: u32) -> i128 {
    let mut total = 0;
    for account in accounts(env).iter() {
        if terminal_delegate(env, &account, ledger) == *target {
            let checkpoints: Vec<BalanceCheckpoint> = env
                .storage()
                .persistent()
                .get(&(BALANCE, account.clone()))
                .unwrap_or_else(|| Vec::new(env));
            total += balance_at(&checkpoints, ledger);
        }
    }
    total
}

fn total_voting_power(env: &Env, ledger: u32) -> i128 {
    accounts(env)
        .iter()
        .map(|account| {
            let checkpoints: Vec<BalanceCheckpoint> = env
                .storage()
                .persistent()
                .get(&(BALANCE, account.clone()))
                .unwrap_or_else(|| Vec::new(env));
            balance_at(&checkpoints, ledger)
        })
        .sum()
}

#[cfg(test)]
mod test;
