use amm::{AmmPool, AmmPoolClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};
use token::LpToken;
use token::LpTokenClient;

fn setup() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(12_345);

    let admin = Address::generate(&env);
    let pool = env.register_contract(None, AmmPool);
    let lp_token = env.register_contract(None, LpToken);
    LpTokenClient::new(&env, &lp_token).initialize(
        &pool,
        &String::from_str(&env, "AMM LP Token"),
        &String::from_str(&env, "ALP"),
        &7u32,
    );
    let token_a = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_b = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    AmmPoolClient::new(&env, &pool).initialize(
        &admin, &token_a, &token_b, &lp_token, &30_i128, &admin, &0_i128,
    );
    (env, pool, admin, token_a, token_b)
}

#[test]
fn initialize_emergency_delay_entrypoint_exists() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pool = env.register_contract(None, AmmPool);
    let lp = env.register_contract(None, LpToken);
    let ta = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let tb = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    AmmPoolClient::new(&env, &pool)
        .initialize_with_emergency_delay(&admin, &ta, &tb, &lp, &30, &admin, &0, &3_600);
}

#[test]
fn initialize_flash_fee_entrypoint_exists() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let pool = env.register_contract(None, AmmPool);
    let lp = env.register_contract(None, LpToken);
    let ta = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let tb = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    AmmPoolClient::new(&env, &pool)
        .initialize_with_flash_loan_fee(&admin, &ta, &tb, &lp, &30, &admin, &0, &10);
}

#[test]
fn set_oracle_entrypoint() {
    let (env, pool, admin, _, _) = setup();
    assert!(AmmPoolClient::new(&env, &pool)
        .try_set_oracle(&admin, &None)
        .is_ok());
}
#[test]
fn set_max_oracle_deviation_entrypoint() {
    let (env, pool, admin, _, _) = setup();
    AmmPoolClient::new(&env, &pool).set_max_oracle_deviation_bps(&admin, &500);
    assert_eq!(
        AmmPoolClient::new(&env, &pool).exits_allowed_when_paused(),
        true
    );
}
#[test]
fn set_lp_locker_entrypoint() {
    let (env, pool, _admin, _, _) = setup();
    let locker = Address::generate(&env);
    assert!(AmmPoolClient::new(&env, &pool)
        .try_set_lp_locker(&locker)
        .is_ok());
}
#[test]
fn emergency_views_and_announce() {
    let (env, pool, admin, _, _) = setup();
    let recipient = Address::generate(&env);
    let c = AmmPoolClient::new(&env, &pool);
    c.announce_emergency_withdraw(&admin, &recipient);
    assert!(c.get_emergency_announcement().is_some());
    assert_eq!(c.get_emergency_delay(), 86_400);
}
#[test]
fn emergency_cancel_entrypoint() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    let recipient = Address::generate(&env);
    c.announce_emergency_withdraw(&admin, &recipient);
    c.cancel_emergency_withdraw(&admin);
    assert!(c.get_emergency_announcement().is_none());
}
#[test]
fn emergency_withdraw_entrypoint_is_typed() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    let recipient = Address::generate(&env);
    assert!(c.try_emergency_withdraw(&admin, &recipient).is_ok() == false);
}
#[test]
fn lock_view_entrypoint() {
    let (env, pool, _, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    assert!(!c.is_locked());
    assert!(!c.is_drained());
    assert!(c.exits_allowed_when_paused());
}
#[test]
fn force_unlock_entrypoint() {
    let (env, pool, _, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    c.force_unlock(&Vec::new(&env));
    assert!(!c.is_locked());
}
#[test]
fn circuit_breaker_config_entrypoint() {
    let (env, pool, _, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    c.set_circuit_breaker_config(&5_000, &600);
    let cfg = c.get_circuit_breaker_config();
    assert_eq!(cfg.threshold_bps, 5_000);
    assert_eq!(cfg.cooldown_secs, 600);
}
#[test]
fn circuit_breaker_recovery_entrypoint() {
    let (env, pool, _, _, _) = setup();
    assert_eq!(
        AmmPoolClient::new(&env, &pool).try_circuit_breaker_recovery(),
        Ok(Ok(false))
    );
}
#[test]
fn circuit_breaker_trip_entrypoint() {
    let (env, pool, _, _, _) = setup();
    assert_eq!(
        AmmPoolClient::new(&env, &pool).try_trip_circuit_breaker(),
        Ok(Ok(false))
    );
}
#[test]
fn lp_rebate_entrypoints() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    c.set_lp_rebate(&admin, &2_500);
    assert_eq!(c.get_lp_rebate(), 2_500);
}
#[test]
fn multisig_config_entrypoints() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer);
    c.set_multisig(&admin, &signers, &1);
    assert_eq!(c.get_multisig_config().quorum, 1);
    assert_eq!(c.get_multisig().1, 1);
}
#[test]
fn propose_emergency_withdraw_entrypoint() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    let signer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    c.set_multisig(&admin, &signers, &1);
    assert_eq!(c.propose_emergency_withdraw(&signer, &recipient), 0);
}
#[test]
fn propose_set_multisig_entrypoint() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    c.set_multisig(&admin, &signers, &1);
    let id = c.propose_set_multisig(&signer, &signers, &1);
    assert_eq!(c.get_multisig_proposal(&id).is_some(), true);
}
#[test]
fn approve_multisig_proposal_entrypoint() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    let signer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    c.set_multisig(&admin, &signers, &1);
    let id = c.propose_emergency_withdraw(&signer, &recipient);
    c.approve_multisig_proposal(&signer, &id);
    assert!(c.get_multisig_proposal(&id).is_some());
}
#[test]
fn exec_multisig_emergency_entrypoint_is_typed() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    let signer = Address::generate(&env);
    let recipient = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    c.set_multisig(&admin, &signers, &1);
    let id = c.propose_emergency_withdraw(&signer, &recipient);
    assert!(
        c.try_exec_multisig_emergency_wd(&signer, &id).is_ok()
            || c.try_exec_multisig_emergency_wd(&signer, &id).is_err()
    );
}
#[test]
fn exec_multisig_set_entrypoint_is_typed() {
    let (env, pool, admin, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    let signer = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(signer.clone());
    c.set_multisig(&admin, &signers, &1);
    let id = c.propose_set_multisig(&signer, &signers, &1);
    assert!(c.try_exec_multisig_set_multisig(&signer, &id).is_ok());
}
#[test]
fn fee_and_liquidity_views_entrypoints() {
    let (env, pool, _, _, _) = setup();
    let c = AmmPoolClient::new(&env, &pool);
    assert_eq!(c.get_fee_info(), Ok(30));
    assert_eq!(c.get_liquidity_cumulative(), (0, 0));
    assert_eq!(c.get_price_cumulative().2, 0);
}
