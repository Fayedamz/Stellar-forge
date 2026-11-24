// Copyright 2024 Stellar-K8s Contributors
// Licensed under the Apache License, Version 2.0.

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn setup() -> (Env, Address, Address, Address, Address) {
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

    (env, contract_id, buyer, token_id, reserve_id)
}

#[test]
fn initialize_sets_curve_configuration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BondingCurveContract);
    let token = Address::generate(&env);
    let reserve = Address::generate(&env);
    let client = BondingCurveContractClient::new(&env, &contract_id);

    client.initialize(&token, &reserve, &5000, &100);

    env.as_contract(&contract_id, || {
        let storage = env.storage().instance();
        assert_eq!(storage.get::<_, Address>(&DataKey::Token), Some(token));
        assert_eq!(
            storage.get::<_, Address>(&DataKey::ReserveToken),
            Some(reserve)
        );
        assert_eq!(storage.get::<_, u32>(&DataKey::ReserveRatio), Some(5000));
        assert_eq!(storage.get::<_, u32>(&DataKey::FeePercentage), Some(100));
    });
    assert_curve_state(&env, &contract_id, 0, 0);
}

#[test]
fn initialize_rejects_duplicate_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BondingCurveContract);
    let token = Address::generate(&env);
    let reserve = Address::generate(&env);
    let client = BondingCurveContractClient::new(&env, &contract_id);

    client.initialize(&token, &reserve, &5000, &100);
    assert!(client.try_initialize(&token, &reserve, &10000, &0).is_err());
    env.as_contract(&contract_id, || {
        assert_eq!(
            env.storage()
                .instance()
                .get::<_, u32>(&DataKey::ReserveRatio),
            Some(5000)
        );
        assert_eq!(
            env.storage()
                .instance()
                .get::<_, u32>(&DataKey::FeePercentage),
            Some(100)
        );
    });
}

#[test]
fn buy_mints_curve_tokens_and_transfers_reserve_tokens() {
    let (env, contract_id, buyer, token_id, reserve_id) = setup();
    let client = BondingCurveContractClient::new(&env, &contract_id);

    client.buy(&buyer, &100, &100);

    let curve_token = token::Client::new(&env, &token_id);
    let reserve_token = token::Client::new(&env, &reserve_id);
    assert_eq!(curve_token.balance(&buyer), 100);
    assert_eq!(reserve_token.balance(&buyer), 9900);
    assert_eq!(reserve_token.balance(&contract_id), 100);
    assert_curve_state(&env, &contract_id, 100, 100);
}

#[test]
fn buy_rejects_excessive_slippage() {
    let (env, contract_id, buyer, token_id, reserve_id) = setup();
    let client = BondingCurveContractClient::new(&env, &contract_id);

    assert!(client.try_buy(&buyer, &100, &99).is_err());

    assert_eq!(token::Client::new(&env, &token_id).balance(&buyer), 0);
    let reserve = token::Client::new(&env, &reserve_id);
    assert_eq!(reserve.balance(&buyer), 10_000);
    assert_eq!(reserve.balance(&contract_id), 0);
    assert_curve_state(&env, &contract_id, 0, 0);
}

#[test]
fn sell_burns_curve_tokens_and_returns_reserve_tokens() {
    let (env, contract_id, buyer, token_id, reserve_id) = setup();
    let client = BondingCurveContractClient::new(&env, &contract_id);
    client.buy(&buyer, &100, &100);

    client.sell(&buyer, &40, &40);

    let curve_token = token::Client::new(&env, &token_id);
    let reserve_token = token::Client::new(&env, &reserve_id);
    assert_eq!(curve_token.balance(&buyer), 60);
    assert_eq!(reserve_token.balance(&buyer), 9940);
    assert_eq!(reserve_token.balance(&contract_id), 60);
    assert_curve_state(&env, &contract_id, 60, 60);
}

#[test]
fn sell_rejects_low_return() {
    let (env, contract_id, buyer, token_id, reserve_id) = setup();
    let client = BondingCurveContractClient::new(&env, &contract_id);
    client.buy(&buyer, &100, &100);

    assert!(client.try_sell(&buyer, &40, &41).is_err());

    assert_eq!(token::Client::new(&env, &token_id).balance(&buyer), 100);
    let reserve = token::Client::new(&env, &reserve_id);
    assert_eq!(reserve.balance(&buyer), 9900);
    assert_eq!(reserve.balance(&contract_id), 100);
    assert_curve_state(&env, &contract_id, 100, 100);
}

fn assert_curve_state(env: &Env, contract_id: &Address, supply: i128, reserve: i128) {
    env.as_contract(contract_id, || {
        let storage = env.storage().instance();
        assert_eq!(storage.get::<_, i128>(&DataKey::TotalSupply), Some(supply));
        assert_eq!(
            storage.get::<_, i128>(&DataKey::ReserveBalance),
            Some(reserve)
        );
    });
}
