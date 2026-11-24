#![no_std]

pub mod math;

use soroban_sdk::{contract, contracterror, contractimpl, symbol_short, Address, Env, Symbol, Vec};

const ADMIN_KEY: Symbol = symbol_short!("Admin");
const ALPHA_KEY: Symbol = symbol_short!("Alpha");
const BUMP_FACTOR_KEY: Symbol = symbol_short!("BumpFact");
const WINDOW_SIZE_KEY: Symbol = symbol_short!("WindowSz");
const RATES_KEY: Symbol = symbol_short!("Rates");
const EMA_KEY: Symbol = symbol_short!("Ema");

const INSTANCE_TTL_THRESHOLD: u32 = 100;
const INSTANCE_TTL_BUMP: u32 = 5000;

const DEFAULT_ALPHA: u64 = 200_000_000;
const DEFAULT_WINDOW_SIZE: u32 = 100;
const DEFAULT_BUMP_FACTOR: u64 = 1_500_000_000;

fn key(_env: &Env, s: Symbol) -> Symbol {
    s
}

#[contracterror]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Error {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    InvalidAlpha = 4,
    InvalidWindowSize = 5,
    InvalidFee = 6,
    InvalidBumpFactor = 7,
}

#[contract]
pub struct GasOracle;

#[contractimpl]
impl GasOracle {
    pub fn initialize(
        env: Env,
        admin: Address,
        alpha: Option<u64>,
        window_size: Option<u32>,
        bump_factor: Option<u64>,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&key(&env, ADMIN_KEY)) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();

        let alpha = alpha.unwrap_or(DEFAULT_ALPHA);
        let window_size = window_size.unwrap_or(DEFAULT_WINDOW_SIZE);
        let bump_factor = bump_factor.unwrap_or(DEFAULT_BUMP_FACTOR);

        if alpha == 0 || alpha >= math::SCALE as u64 {
            return Err(Error::InvalidAlpha);
        }
        if window_size == 0 {
            return Err(Error::InvalidWindowSize);
        }
        if bump_factor == 0 {
            return Err(Error::InvalidBumpFactor);
        }

        env.storage().instance().set(&key(&env, ADMIN_KEY), &admin);
        env.storage().instance().set(&key(&env, ALPHA_KEY), &alpha);
        env.storage()
            .instance()
            .set(&key(&env, WINDOW_SIZE_KEY), &window_size);
        env.storage()
            .instance()
            .set(&key(&env, BUMP_FACTOR_KEY), &bump_factor);
        let empty_rates: Vec<u64> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&key(&env, RATES_KEY), &empty_rates);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_BUMP);
        Ok(())
    }

    pub fn update(env: Env, fee: u64) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&key(&env, ADMIN_KEY))
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        if fee == 0 {
            return Err(Error::InvalidFee);
        }

        let alpha: u64 = env
            .storage()
            .instance()
            .get(&key(&env, ALPHA_KEY))
            .ok_or(Error::NotInitialized)?;
        let window_size: u32 = env
            .storage()
            .instance()
            .get(&key(&env, WINDOW_SIZE_KEY))
            .ok_or(Error::NotInitialized)?;

        let mut rates: Vec<u64> = env
            .storage()
            .instance()
            .get(&key(&env, RATES_KEY))
            .unwrap_or_else(|| Vec::new(&env));
        rates.push_back(fee);
        while rates.len() > window_size {
            rates.remove(0);
        }
        env.storage().instance().set(&key(&env, RATES_KEY), &rates);

        let current_ema: u128 = match env.storage().instance().get(&key(&env, EMA_KEY)) {
            Some(v) => v,
            None => {
                let init = math::initial_ema(fee);
                env.storage().instance().set(&key(&env, EMA_KEY), &init);
                env.storage()
                    .instance()
                    .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_BUMP);
                return Ok(());
            }
        };

        let _ = alpha;
        let new_ema = math::compute_ema(alpha, current_ema, fee);
        env.storage().instance().set(&key(&env, EMA_KEY), &new_ema);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_BUMP);
        Ok(())
    }

    pub fn get_ema(env: Env) -> Result<u64, Error> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_BUMP);
        let ema: u128 = env
            .storage()
            .instance()
            .get(&key(&env, EMA_KEY))
            .ok_or(Error::NotInitialized)?;
        Ok(math::ema_scaled_to_units(ema))
    }

    pub fn get_suggested_fee(env: Env) -> Result<u64, Error> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_BUMP);
        let ema: u128 = env
            .storage()
            .instance()
            .get(&key(&env, EMA_KEY))
            .ok_or(Error::NotInitialized)?;
        let bump_factor: u64 = env
            .storage()
            .instance()
            .get(&key(&env, BUMP_FACTOR_KEY))
            .ok_or(Error::NotInitialized)?;
        Ok(math::apply_bump_factor(ema, bump_factor))
    }

    pub fn get_recent_rates(env: Env) -> Result<Vec<u64>, Error> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_BUMP);
        env.storage()
            .instance()
            .get(&key(&env, RATES_KEY))
            .ok_or(Error::NotInitialized)
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, GasOracle, GasOracleClient};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    fn setup() -> GasOracleClient<'static> {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let contract_id = env.register(GasOracle, ());
        let client = GasOracleClient::new(&env, &contract_id);
        client.initialize(&admin, &None, &None, &None);
        client
    }

    #[test]
    fn initialize_is_idempotent_guarded() {
        let client = setup();
        let other_admin = Address::generate(&client.env);
        let res = client.try_initialize(&other_admin, &None, &None, &None);
        assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
        let rates = client.get_recent_rates();
        assert_eq!(rates.len(), 0);
    }

    #[test]
    fn update_tracks_ema() {
        let client = setup();
        client.update(&100);
        client.update(&200);
        let ema = client.get_ema();
        assert!(ema >= 100);
        let suggested = client.get_suggested_fee();
        assert!(suggested >= ema);
        let rates = client.get_recent_rates();
        assert_eq!(rates.len(), 2);
    }

    #[test]
    fn zero_fee_rejected() {
        let client = setup();
        let res = client.try_update(&0);
        assert_eq!(res, Err(Ok(Error::InvalidFee)));
    }
}
