//! # IotDeviceManagement Events Module
//!
//! Standardized event emissions for the iot_device_management contract.
//! Topic naming convention: (IDM, ACTION)

#![allow(dead_code)]

use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum EventType {
    Initialized,
    Action,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum OperationCategory {
    Administrative,
    Operations,
}

#[derive(Clone)]
#[contracttype]
pub struct IotDeviceManagementEventData {
    pub user: Address,
    pub action: String,
}

#[derive(Clone)]
#[contracttype]
pub struct IotDeviceManagementEvent {
    pub event_type: EventType,
    pub category: OperationCategory,
    pub timestamp: u64,
    pub user_id: Address,
    pub block_height: u64,
    pub data: IotDeviceManagementEventData,
}

/// Emitted when initialize is called.
pub fn emit_initialize(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Initialized,
        category: OperationCategory::Administrative,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "initialize"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("INIT")), event);
}

/// Emitted when pause is called.
pub fn emit_pause(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "pause"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("PAUSE")), event);
}

/// Emitted when unpause is called.
pub fn emit_unpause(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "unpause"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("UNPAUSE")), event);
}

/// Emitted when set_role is called.
pub fn emit_set_role(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_role"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("SET_ROLE")), event);
}

/// Emitted when register_manufacturer is called.
pub fn emit_register_manufacturer(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_manufacturer"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("REGISTER_")), event);
}

/// Emitted when deactivate_manufacturer is called.
pub fn emit_deactivate_manufacturer(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "deactivate_manufacturer"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("DEACTIVAT")), event);
}

/// Emitted when register_device is called.
pub fn emit_register_device(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "register_device"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("REGISTER_")), event);
}

/// Emitted when activate_device is called.
pub fn emit_activate_device(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "activate_device"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("ACTIVATE_")), event);
}

/// Emitted when suspend_device is called.
pub fn emit_suspend_device(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "suspend_device"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("SUSPEND_D")), event);
}

/// Emitted when decommission_device is called.
pub fn emit_decommission_device(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "decommission_device"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("DECOMMISS")), event);
}

/// Emitted when publish_firmware is called.
pub fn emit_publish_firmware(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "publish_firmware"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("PUBLISH_F")), event);
}

/// Emitted when approve_firmware is called.
pub fn emit_approve_firmware(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "approve_firmware"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("APPROVE_F")), event);
}

/// Emitted when reject_firmware is called.
pub fn emit_reject_firmware(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "reject_firmware"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("REJECT_FI")), event);
}

/// Emitted when update_device_firmware is called.
pub fn emit_update_device_firmware(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "update_device_firmware"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("UPDATE_DE")), event);
}

/// Emitted when submit_heartbeat is called.
pub fn emit_submit_heartbeat(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "submit_heartbeat"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("SUBMIT_HE")), event);
}

/// Emitted when set_heartbeat_interval is called.
pub fn emit_set_heartbeat_interval(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_heartbeat_interval"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("SET_HEART")), event);
}

/// Emitted when create_comm_channel is called.
pub fn emit_create_comm_channel(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "create_comm_channel"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("CREATE_CO")), event);
}

/// Emitted when rotate_encryption_key is called.
pub fn emit_rotate_encryption_key(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "rotate_encryption_key"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("ROTATE_EN")), event);
}

/// Emitted when rotate_device_key is called.
pub fn emit_rotate_device_key(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "rotate_device_key"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("ROTATE_DE")), event);
}

/// Emitted when set_key_rotation_interval is called.
pub fn emit_set_key_rotation_interval(env: &Env, caller: &Address) {
    let event = IotDeviceManagementEvent {
        event_type: EventType::Action,
        category: OperationCategory::Operations,
        timestamp: env.ledger().timestamp(),
        user_id: caller.clone(),
        block_height: env.ledger().sequence() as u64,
        data: IotDeviceManagementEventData {
            user: caller.clone(),
            action: String::from_str(env, "set_key_rotation_interval"),
        },
    };
    env.events()
        .publish((symbol_short!("IDM"), symbol_short!("SET_KEY_R")), event);
}
