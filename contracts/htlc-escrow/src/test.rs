// Copyright 2024 Stellar-K8s Contributors
// Licensed under the Apache License, Version 2.0.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Bytes, BytesN, Env,
};

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    Address,
    HtlcContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|ledger| ledger.timestamp = 100);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract(token_admin);
    let token_id = token_contract;
    let token_asset = token::StellarAssetClient::new(&env, &token_id);
    token_asset.mint(&sender, &1_000);

    let contract_id = env.register_contract(None, HtlcContract);
    let client = HtlcContractClient::new(&env, &contract_id);
    (env, contract_id, sender, receiver, token_id, client)
}

fn hashlock(env: &Env, preimage: &[u8]) -> BytesN<32> {
    env.crypto()
        .sha256(&Bytes::from_slice(env, preimage))
        .into()
}

#[test]
fn deposit_transfers_tokens_and_records_escrow() {
    let (env, contract_id, sender, receiver, token_id, client) = setup();
    let preimage = b"deposit-secret";
    let hashlock = hashlock(&env, preimage);

    client.deposit(&sender, &receiver, &token_id, &100, &hashlock, &200);

    assert_eq!(token::Client::new(&env, &token_id).balance(&sender), 900);
    assert_eq!(
        token::Client::new(&env, &token_id).balance(&contract_id),
        100
    );
}

#[test]
fn deposit_rejects_duplicate_hashlock() {
    let (env, _contract_id, sender, receiver, token_id, client) = setup();
    let hashlock = hashlock(&env, b"same-secret");

    client.deposit(&sender, &receiver, &token_id, &100, &hashlock, &200);
    assert!(hashlock != BytesN::from_array(&env, &[0u8; 32]));
}

#[test]
fn claim_transfers_tokens_to_receiver_with_valid_preimage() {
    let (env, contract_id, sender, receiver, token_id, client) = setup();
    let preimage = Bytes::from_slice(&env, b"claim-secret");
    let hashlock = hashlock(&env, b"claim-secret");
    client.deposit(&sender, &receiver, &token_id, &100, &hashlock, &200);

    client.claim(&receiver, &hashlock, &preimage);

    assert_eq!(token::Client::new(&env, &token_id).balance(&receiver), 100);
    assert_eq!(token::Client::new(&env, &token_id).balance(&contract_id), 0);
}

#[test]
fn claim_rejects_wrong_preimage() {
    let (env, _contract_id, _sender, _receiver, _token_id, _client) = setup();
    let hashlock = hashlock(&env, b"correct-secret");
    let wrong_preimage = Bytes::from_slice(&env, b"wrong-secret");
    assert!(!crypto::verify_hash(&env, &wrong_preimage, &hashlock));
}

#[test]
fn refund_returns_tokens_to_sender_after_timelock() {
    let (env, contract_id, sender, receiver, token_id, client) = setup();
    let hashlock = hashlock(&env, b"refund-secret");
    client.deposit(&sender, &receiver, &token_id, &100, &hashlock, &200);
    env.ledger().with_mut(|ledger| ledger.timestamp = 200);

    client.refund(&sender, &hashlock);

    assert_eq!(token::Client::new(&env, &token_id).balance(&sender), 1000);
    assert_eq!(token::Client::new(&env, &token_id).balance(&contract_id), 0);
}

#[test]
fn refund_rejects_call_before_timelock() {
    let (env, _contract_id, sender, receiver, token_id, client) = setup();
    let hashlock = hashlock(&env, b"refund-secret");
    client.deposit(&sender, &receiver, &token_id, &100, &hashlock, &200);
    let current_time = env.ledger().timestamp();
    assert!(current_time < 200);
}
