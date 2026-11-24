// Copyright 2024 Stellar-K8s Contributors
// Licensed under the Apache License, Version 2.0.

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn setup() -> (
    Env,
    Address,
    Address,
    Address,
    Address,
    BondingCurveContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let buyer = Address::generate(&env);
    let contract_id = env.register_contract(None, BondingCurveContract);
    let token_id = env.register_stellar_asset_contract(contract_id.clone());
    let reserve_id = env.register_stellar_asset_contract(Address::generate(&env));
    let reserve_admin = token::StellarAssetClient::new(&env, &reserve_id);
    reserve_admin.mint(&buyer, &10_000);

    let client = BondingCurveContractClient::new(&env, &contract_id);
    client.initialize(&token_id, &reserve_id, &10000, &0);

    (env, contract_id, buyer, token_id, reserve_id, client)
}

#[test]
fn initialize_sets_curve_configuration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BondingCurveContract);
    let token = Address::generate(&env);
    let reserve = Address::generate(&env);
    let client = BondingCurveContractClient::new(&env, &contract_id);

    client.initialize(&token, &reserve, &5000, &100);
}

#[test]
fn initialize_rejects_duplicate_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BondingCurveContract);
    let token = Address::generate(&env);
    let reserve = Address::generate(&env);
    let client = BondingCurveContractClient::new(&env, &contract_id);

    client.initialize(&token, &reserve, &5000, &100);
    assert_ne!(token, reserve);
}

#[test]
fn buy_mints_curve_tokens_and_transfers_reserve_tokens() {
    let (env, _contract_id, buyer, token_id, reserve_id, client) = setup();

    client.buy(&buyer, &100, &100);

    let curve_token = token::Client::new(&env, &token_id);
    let reserve_token = token::Client::new(&env, &reserve_id);
    assert_eq!(curve_token.balance(&buyer), 100);
    assert_eq!(reserve_token.balance(&buyer), 9900);
}

#[test]
fn buy_rejects_excessive_slippage() {
    let cost = curve::calculate_purchase_return(0, 0, 10000, 100);
    assert_eq!(cost, 100);
    assert!(cost > 99);
}

#[test]
fn sell_burns_curve_tokens_and_returns_reserve_tokens() {
    let (env, _contract_id, buyer, token_id, reserve_id, client) = setup();
    client.buy(&buyer, &100, &100);

    client.sell(&buyer, &40, &40);

    let curve_token = token::Client::new(&env, &token_id);
    let reserve_token = token::Client::new(&env, &reserve_id);
    assert_eq!(curve_token.balance(&buyer), 60);
    assert_eq!(reserve_token.balance(&buyer), 9940);
}

#[test]
fn sell_rejects_low_return() {
    let return_amount = curve::calculate_sale_return(100, 100, 10000, 40);
    assert_eq!(return_amount, 40);
    assert!(return_amount < 41);
}
