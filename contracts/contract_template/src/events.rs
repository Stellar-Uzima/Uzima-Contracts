use soroban_sdk::{symbol_short, Address, Env, String};

pub fn emit_initialized(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("init"),), (admin.clone(),));
}

pub fn emit_admin_transferred(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (symbol_short!("adm_xfer"),),
        (old_admin.clone(), new_admin.clone()),
    );
}

pub fn emit_data_updated(env: &Env, caller: &Address, data: &String) {
    env.events()
        .publish((symbol_short!("upd_data"),), (caller.clone(), data.clone()));
}
