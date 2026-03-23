#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, IntoVal,
    String, Vec,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    Node(BytesN<32>), // node_id
    Task(BytesN<32>), // task_id
    NodeList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CloudProvider {
    AWSNitro,
    IntelSGX,
    GCPConfidentialSpace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EnclaveStatus {
    PendingRegistration,
    Active,
    Compromised,
    Offline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct EnclaveNode {
    pub provider: CloudProvider,
    pub quote: Bytes, // Attestation quote
    pub public_key: BytesN<32>,
    pub status: EnclaveStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum TaskStatus {
    Submitted,
    Processing,
    Completed,
    Failed,
    FallbackMPC,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ProcessingTask {
    pub submitter: Address,
    pub payload_hash: BytesN<32>,
    pub status: TaskStatus,
    pub result: Option<Bytes>,
    pub assigned_node: Bytes, // Empty Bytes implies None
    pub require_zk_proof: bool,
}

#[contract]
pub struct SecureEnclaveContract;

#[contractimpl]
impl SecureEnclaveContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn register_enclave(
        env: Env,
        caller: Address,
        node_id: BytesN<32>,
        provider: CloudProvider,
        quote: Bytes,
        public_key: BytesN<32>,
    ) {
        caller.require_auth();

        let key = DataKey::Node(node_id.clone());
        if env.storage().persistent().has(&key) {
            panic!("node already registered");
        }

        let node = EnclaveNode {
            provider,
            quote,
            public_key,
            status: EnclaveStatus::PendingRegistration,
        };
        env.storage().persistent().set(&key, &node);

        let mut node_list: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::NodeList)
            .unwrap_or(Vec::new(&env));
        node_list.push_back(node_id);
        env.storage().instance().set(&DataKey::NodeList, &node_list);
    }

    pub fn verify_attestation(env: Env, admin: Address, node_id: BytesN<32>, is_valid: bool) {
        admin.require_auth();
        let expected_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != expected_admin {
            panic!("unauthorized");
        }

        let key = DataKey::Node(node_id.clone());
        let mut node: EnclaveNode = env.storage().persistent().get(&key).unwrap();

        if is_valid {
            node.status = EnclaveStatus::Active;
        } else {
            node.status = EnclaveStatus::Compromised;
        }
        env.storage().persistent().set(&key, &node);
    }

    pub fn submit_task(
        env: Env,
        submitter: Address,
        task_id: BytesN<32>,
        payload_hash: BytesN<32>,
        require_zk_proof: bool,
    ) {
        submitter.require_auth();
        let key = DataKey::Task(task_id.clone());
        if env.storage().persistent().has(&key) {
            panic!("task already exists");
        }
        let task = ProcessingTask {
            submitter,
            payload_hash,
            status: TaskStatus::Submitted,
            result: None,
            assigned_node: Bytes::new(&env),
            require_zk_proof,
        };
        env.storage().persistent().set(&key, &task);
    }

    pub fn assign_task(env: Env, admin: Address, task_id: BytesN<32>, node_id: BytesN<32>) {
        admin.require_auth();
        let expected_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != expected_admin {
            panic!("unauthorized");
        }

        let node_key = DataKey::Node(node_id.clone());
        let node: EnclaveNode = env.storage().persistent().get(&node_key).unwrap();
        if node.status != EnclaveStatus::Active {
            panic!("node not active");
        }

        let task_key = DataKey::Task(task_id.clone());
        let mut task: ProcessingTask = env.storage().persistent().get(&task_key).unwrap();
        task.assigned_node = node_id.into(); // Convert BytesN<32> to Bytes
        task.status = TaskStatus::Processing;
        env.storage().persistent().set(&task_key, &task);
    }

    pub fn complete_task(
        env: Env,
        node_address: Address,
        task_id: BytesN<32>,
        result: Bytes,
        zk_proof: Option<Bytes>,
    ) {
        node_address.require_auth();

        let task_key = DataKey::Task(task_id.clone());
        let mut task: ProcessingTask = env.storage().persistent().get(&task_key).unwrap();

        if task.require_zk_proof {
            if zk_proof.is_none() {
                panic!("zk proof required");
            }
        }

        task.status = TaskStatus::Completed;
        task.result = Some(result);
        env.storage().persistent().set(&task_key, &task);
    }

    pub fn fallback_to_mpc(env: Env, admin: Address, task_id: BytesN<32>, mpc_manager_id: Address) {
        admin.require_auth();
        let expected_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != expected_admin {
            panic!("unauthorized");
        }

        let task_key = DataKey::Task(task_id.clone());
        let mut task: ProcessingTask = env.storage().persistent().get(&task_key).unwrap();

        // Ensure not already completed
        if task.status == TaskStatus::Completed {
            panic!("task already completed");
        }

        let mut args: Vec<soroban_sdk::Val> = Vec::new(&env);
        args.push_back(task_id.into_val(&env));
        args.push_back(task.payload_hash.into_val(&env));

        // Use invoke_contract to call the fallback
        env.invoke_contract::<soroban_sdk::Val>(&mpc_manager_id, &symbol_short!("req_mpc"), args);

        task.status = TaskStatus::FallbackMPC;
        env.storage().persistent().set(&task_key, &task);
    }
}

#[cfg(test)]
mod test;
