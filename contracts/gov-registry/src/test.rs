use super::{GovRegistry, GovRegistryClient, ProposalState, Vote};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Symbol,
};

#[test]
fn delegation_chain_uses_snapshot_balances() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    let id = env.register(GovRegistry, (&admin,));
    let client = GovRegistryClient::new(&env, &id);
    env.mock_all_auths();
    client.mint(&a, &10);
    client.mint(&b, &20);
    client.mint(&c, &30);
    client.delegate(&a, &Some(b.clone()));
    client.delegate(&b, &Some(c.clone()));
    assert_eq!(client.voting_power(&c, &env.ledger().sequence()), 60);
}

#[test]
fn proposal_votes_use_creation_snapshot_and_finalize_quorum() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let id = env.register(GovRegistry, (&admin,));
    let client = GovRegistryClient::new(&env, &id);
    env.mock_all_auths();
    client.mint(&a, &60);
    client.mint(&b, &40);
    let proposal = client.propose(&a, &Symbol::new(&env, "upgrade"), &2, &6_000, &50);
    client.transfer(&a, &b, &60);
    client.vote(&a, &proposal, &Vote::For);
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + 2);
    client.finalize(&proposal);
    assert_eq!(
        client.get_proposal(&proposal).unwrap().state,
        ProposalState::Passed
    );
}

#[test]
fn balance_checkpoints_preserve_history_and_coalesce_same_ledger_writes() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let holder = Address::generate(&env);
    let recipient = Address::generate(&env);
    let id = env.register(GovRegistry, (&admin,));
    let client = GovRegistryClient::new(&env, &id);
    env.mock_all_auths();

    client.mint(&holder, &10);
    client.mint(&holder, &15);
    let checkpoint_ledger = env.ledger().sequence();
    assert_eq!(client.voting_power(&holder, &checkpoint_ledger), 25);

    env.ledger().set_sequence_number(checkpoint_ledger + 1);
    client.transfer(&holder, &recipient, &7);

    assert_eq!(client.voting_power(&holder, &checkpoint_ledger), 25);
    assert_eq!(client.voting_power(&holder, &env.ledger().sequence()), 18);
}
