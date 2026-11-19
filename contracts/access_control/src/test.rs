use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env,
};

#[contract]
struct AuthProbe;

#[contractimpl]
impl AuthProbe {
    pub fn privileged(env: Env, caller: Address) -> bool {
        caller.require_auth();
        true
    }

    pub fn unprivileged(_env: Env) -> bool {
        true
    }
}

#[test]
fn explicit_mock_auths_authorize_privileged_call() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthProbe);
    let client = AuthProbeClient::new(&env, &contract_id);
    let caller = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &caller,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "privileged",
            args: (),
            sub_invokes: &[],
        },
    }]);

    assert!(client.privileged(&caller));
}

#[test]
fn missing_explicit_auth_rejects_privileged_call() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthProbe);
    let client = AuthProbeClient::new(&env, &contract_id);
    let caller = Address::generate(&env);

    env.mock_auths(&[]);

    assert!(client.try_privileged(&caller).is_err());
}

#[test]
fn unprivileged_call_does_not_require_auth() {
    let env = Env::default();
    let contract_id = env.register_contract(None, AuthProbe);
    let client = AuthProbeClient::new(&env, &contract_id);

    env.mock_auths(&[]);

    assert!(client.unprivileged());
}
