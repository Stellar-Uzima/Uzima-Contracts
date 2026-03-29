#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ManufacturerNotFound = 4,
    MedicationNotFound = 5,
    BatchNotFound = 6,
    ShipmentNotFound = 7,
    InvalidInput = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BatchStatus {
    Manufactured = 0,
    InTransit = 1,
    Delivered = 2,
    Recalled = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ShipmentStatus {
    Created = 0,
    InTransit = 1,
    Delivered = 2,
    Flagged = 3,
}

#[derive(Clone)]
#[contracttype]
pub struct Manufacturer {
    pub id: u64,
    pub operator: Address,
    pub name: String,
    pub license_number: String,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Medication {
    pub id: u64,
    pub manufacturer_id: u64,
    pub name: String,
    pub ndc: String,
    pub requires_cold_chain: bool,
    pub min_temp_c: i32,
    pub max_temp_c: i32,
    pub regulatory_region: String,
}

#[derive(Clone)]
#[contracttype]
pub struct Batch {
    pub id: u64,
    pub medication_id: u64,
    pub lot_number: String,
    pub quantity: u32,
    pub auth_hash: BytesN<32>,
    pub manufactured_at: u64,
    pub expires_at: u64,
    pub current_owner: Address,
    pub status: BatchStatus,
    pub compliance_ok: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Shipment {
    pub id: u64,
    pub batch_id: u64,
    pub from: Address,
    pub to: Address,
    pub carrier_ref: String,
    pub status: ShipmentStatus,
    pub latest_temp_c: i32,
    pub latest_humidity_bps: u32,
    pub latitude_e6: i32,
    pub longitude_e6: i32,
    pub compliance_ok: bool,
    pub created_at: u64,
    pub delivered_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct InventorySnapshot {
    pub owner: Address,
    pub batch_count: u32,
    pub total_units: u32,
    pub cold_chain_violations: u32,
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct InventoryRecommendation {
    pub owner: Address,
    pub available_units: u32,
    pub forecast_units: u32,
    pub reorder_needed: bool,
    pub recommended_reorder_units: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    ManufacturerCount,
    Manufacturer(u64),
    MedicationCount,
    Medication(u64),
    BatchCount,
    Batch(u64),
    ShipmentCount,
    Shipment(u64),
}

#[contract]
pub struct PharmaSupplyChainContract;

#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl PharmaSupplyChainContract {
    fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if admin != *caller {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    fn next_counter(env: &Env, key: &DataKey) -> u64 {
        let current: u64 = env.storage().instance().get(key).unwrap_or(0);
        let next = current.saturating_add(1);
        env.storage().instance().set(key, &next);
        next
    }

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ManufacturerCount, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::MedicationCount, &0u64);
        env.storage().instance().set(&DataKey::BatchCount, &0u64);
        env.storage().instance().set(&DataKey::ShipmentCount, &0u64);
        Ok(())
    }

    pub fn register_manufacturer(
        env: Env,
        admin: Address,
        operator: Address,
        name: String,
        license_number: String,
    ) -> Result<u64, Error> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        if name.is_empty() || license_number.is_empty() {
            return Err(Error::InvalidInput);
        }

        let id = Self::next_counter(&env, &DataKey::ManufacturerCount);
        let manufacturer = Manufacturer {
            id,
            operator,
            name,
            license_number,
            active: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Manufacturer(id), &manufacturer);
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_medication(
        env: Env,
        caller: Address,
        manufacturer_id: u64,
        name: String,
        ndc: String,
        requires_cold_chain: bool,
        min_temp_c: i32,
        max_temp_c: i32,
        regulatory_region: String,
    ) -> Result<u64, Error> {
        caller.require_auth();
        let manufacturer: Manufacturer = env
            .storage()
            .persistent()
            .get(&DataKey::Manufacturer(manufacturer_id))
            .ok_or(Error::ManufacturerNotFound)?;
        if manufacturer.operator != caller || name.is_empty() || ndc.is_empty() {
            return Err(Error::Unauthorized);
        }
        if requires_cold_chain && min_temp_c > max_temp_c {
            return Err(Error::InvalidInput);
        }

        let id = Self::next_counter(&env, &DataKey::MedicationCount);
        let medication = Medication {
            id,
            manufacturer_id,
            name,
            ndc,
            requires_cold_chain,
            min_temp_c,
            max_temp_c,
            regulatory_region,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Medication(id), &medication);
        Ok(id)
    }

    pub fn create_batch(
        env: Env,
        caller: Address,
        medication_id: u64,
        lot_number: String,
        quantity: u32,
        auth_hash: BytesN<32>,
        expires_at: u64,
    ) -> Result<u64, Error> {
        caller.require_auth();
        if lot_number.is_empty() || quantity == 0 {
            return Err(Error::InvalidInput);
        }
        let medication: Medication = env
            .storage()
            .persistent()
            .get(&DataKey::Medication(medication_id))
            .ok_or(Error::MedicationNotFound)?;
        let manufacturer: Manufacturer = env
            .storage()
            .persistent()
            .get(&DataKey::Manufacturer(medication.manufacturer_id))
            .ok_or(Error::ManufacturerNotFound)?;
        if manufacturer.operator != caller {
            return Err(Error::Unauthorized);
        }

        let id = Self::next_counter(&env, &DataKey::BatchCount);
        let batch = Batch {
            id,
            medication_id,
            lot_number,
            quantity,
            auth_hash,
            manufactured_at: env.ledger().timestamp(),
            expires_at,
            current_owner: caller,
            status: BatchStatus::Manufactured,
            compliance_ok: true,
        };
        env.storage().persistent().set(&DataKey::Batch(id), &batch);
        Ok(id)
    }

    pub fn verify_batch_authenticity(
        env: Env,
        batch_id: u64,
        auth_hash: BytesN<32>,
    ) -> Result<bool, Error> {
        let batch: Batch = env
            .storage()
            .persistent()
            .get(&DataKey::Batch(batch_id))
            .ok_or(Error::BatchNotFound)?;
        Ok(batch.auth_hash == auth_hash)
    }

    pub fn create_shipment(
        env: Env,
        caller: Address,
        batch_id: u64,
        to: Address,
        carrier_ref: String,
    ) -> Result<u64, Error> {
        caller.require_auth();
        if carrier_ref.is_empty() {
            return Err(Error::InvalidInput);
        }
        let mut batch: Batch = env
            .storage()
            .persistent()
            .get(&DataKey::Batch(batch_id))
            .ok_or(Error::BatchNotFound)?;
        if batch.current_owner != caller {
            return Err(Error::Unauthorized);
        }

        batch.status = BatchStatus::InTransit;
        env.storage()
            .persistent()
            .set(&DataKey::Batch(batch_id), &batch);

        let shipment_id = Self::next_counter(&env, &DataKey::ShipmentCount);
        let shipment = Shipment {
            id: shipment_id,
            batch_id,
            from: caller,
            to,
            carrier_ref,
            status: ShipmentStatus::InTransit,
            latest_temp_c: 0,
            latest_humidity_bps: 0,
            latitude_e6: 0,
            longitude_e6: 0,
            compliance_ok: true,
            created_at: env.ledger().timestamp(),
            delivered_at: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Shipment(shipment_id), &shipment);
        env.events()
            .publish((symbol_short!("SHIP"),), (shipment_id, batch_id));

        Ok(shipment_id)
    }

    pub fn log_condition_data(
        env: Env,
        caller: Address,
        shipment_id: u64,
        temperature_c: i32,
        humidity_bps: u32,
        latitude_e6: i32,
        longitude_e6: i32,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut shipment: Shipment = env
            .storage()
            .persistent()
            .get(&DataKey::Shipment(shipment_id))
            .ok_or(Error::ShipmentNotFound)?;
        if shipment.from != caller && shipment.to != caller {
            return Err(Error::Unauthorized);
        }

        let mut batch: Batch = env
            .storage()
            .persistent()
            .get(&DataKey::Batch(shipment.batch_id))
            .ok_or(Error::BatchNotFound)?;
        let medication: Medication = env
            .storage()
            .persistent()
            .get(&DataKey::Medication(batch.medication_id))
            .ok_or(Error::MedicationNotFound)?;

        shipment.latest_temp_c = temperature_c;
        shipment.latest_humidity_bps = humidity_bps;
        shipment.latitude_e6 = latitude_e6;
        shipment.longitude_e6 = longitude_e6;

        if medication.requires_cold_chain
            && (temperature_c < medication.min_temp_c || temperature_c > medication.max_temp_c)
        {
            shipment.compliance_ok = false;
            shipment.status = ShipmentStatus::Flagged;
            batch.compliance_ok = false;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Shipment(shipment_id), &shipment);
        env.storage()
            .persistent()
            .set(&DataKey::Batch(batch.id), &batch);
        Ok(shipment.compliance_ok)
    }

    pub fn complete_shipment(
        env: Env,
        caller: Address,
        shipment_id: u64,
        verified: bool,
    ) -> Result<bool, Error> {
        caller.require_auth();
        let mut shipment: Shipment = env
            .storage()
            .persistent()
            .get(&DataKey::Shipment(shipment_id))
            .ok_or(Error::ShipmentNotFound)?;
        if shipment.to != caller {
            return Err(Error::Unauthorized);
        }

        let mut batch: Batch = env
            .storage()
            .persistent()
            .get(&DataKey::Batch(shipment.batch_id))
            .ok_or(Error::BatchNotFound)?;
        shipment.delivered_at = env.ledger().timestamp();
        shipment.status = if verified && shipment.compliance_ok {
            ShipmentStatus::Delivered
        } else {
            ShipmentStatus::Flagged
        };

        batch.current_owner = caller;
        batch.status = if shipment.status == ShipmentStatus::Delivered {
            BatchStatus::Delivered
        } else {
            BatchStatus::InTransit
        };

        env.storage()
            .persistent()
            .set(&DataKey::Shipment(shipment_id), &shipment);
        env.storage()
            .persistent()
            .set(&DataKey::Batch(batch.id), &batch);
        Ok(shipment.status == ShipmentStatus::Delivered)
    }

    pub fn run_compliance_check(env: Env, batch_id: u64) -> Result<bool, Error> {
        let batch: Batch = env
            .storage()
            .persistent()
            .get(&DataKey::Batch(batch_id))
            .ok_or(Error::BatchNotFound)?;
        Ok(batch.compliance_ok && batch.expires_at > env.ledger().timestamp())
    }

    pub fn get_inventory_snapshot(env: Env, owner: Address) -> InventorySnapshot {
        let batch_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::BatchCount)
            .unwrap_or(0);
        let mut owned_batches = 0u32;
        let mut total_units = 0u32;
        let mut cold_chain_violations = 0u32;

        for batch_id in 1..=batch_count {
            if let Some(batch) = env
                .storage()
                .persistent()
                .get::<DataKey, Batch>(&DataKey::Batch(batch_id))
            {
                if batch.current_owner == owner {
                    owned_batches = owned_batches.saturating_add(1);
                    total_units = total_units.saturating_add(batch.quantity);
                    if !batch.compliance_ok {
                        cold_chain_violations = cold_chain_violations.saturating_add(1);
                    }
                }
            }
        }

        InventorySnapshot {
            owner,
            batch_count: owned_batches,
            total_units,
            cold_chain_violations,
            last_updated: env.ledger().timestamp(),
        }
    }

    pub fn optimize_inventory(
        env: Env,
        owner: Address,
        forecast_units: u32,
    ) -> InventoryRecommendation {
        let snapshot = Self::get_inventory_snapshot(env.clone(), owner.clone());
        let reorder_needed = snapshot.total_units < forecast_units;
        InventoryRecommendation {
            owner,
            available_units: snapshot.total_units,
            forecast_units,
            reorder_needed,
            recommended_reorder_units: if reorder_needed {
                forecast_units.saturating_sub(snapshot.total_units)
            } else {
                0
            },
        }
    }

    pub fn get_batch(env: Env, batch_id: u64) -> Result<Batch, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Batch(batch_id))
            .ok_or(Error::BatchNotFound)
    }

    pub fn get_shipment(env: Env, shipment_id: u64) -> Result<Shipment, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Shipment(shipment_id))
            .ok_or(Error::ShipmentNotFound)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_batch_tracking_shipment_and_inventory() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PharmaSupplyChainContract);
        let client = PharmaSupplyChainContractClient::new(&env, &contract_id);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let manufacturer_operator = Address::generate(&env);
        let distributor = Address::generate(&env);

        client.initialize(&admin);
        let manufacturer_id = client.register_manufacturer(
            &admin,
            &manufacturer_operator,
            &String::from_str(&env, "Uzima Pharma"),
            &String::from_str(&env, "FDA-001"),
        );
        let medication_id = client.register_medication(
            &manufacturer_operator,
            &manufacturer_id,
            &String::from_str(&env, "Cold Vaccine"),
            &String::from_str(&env, "NDC-001"),
            &true,
            &2i32,
            &8i32,
            &String::from_str(&env, "FDA"),
        );
        let batch_id = client.create_batch(
            &manufacturer_operator,
            &medication_id,
            &String::from_str(&env, "LOT-1"),
            &100u32,
            &BytesN::from_array(&env, &[3u8; 32]),
            &9_999_999u64,
        );
        assert!(client.verify_batch_authenticity(&batch_id, &BytesN::from_array(&env, &[3u8; 32])));

        let shipment_id = client.create_shipment(
            &manufacturer_operator,
            &batch_id,
            &distributor,
            &String::from_str(&env, "SHIP-REF-1"),
        );
        assert!(client.log_condition_data(
            &manufacturer_operator,
            &shipment_id,
            &5i32,
            &6500u32,
            &123,
            &456
        ));
        assert!(client.complete_shipment(&distributor, &shipment_id, &true));

        let batch = client.get_batch(&batch_id);
        assert_eq!(batch.current_owner, distributor);
        let inventory = client.get_inventory_snapshot(&distributor);
        assert_eq!(inventory.total_units, 100u32);

        let recommendation = client.optimize_inventory(&distributor, &140u32);
        assert!(recommendation.reorder_needed);
        assert_eq!(recommendation.recommended_reorder_units, 40u32);
    }

    #[test]
    fn test_cold_chain_violation_flags_shipment() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PharmaSupplyChainContract);
        let client = PharmaSupplyChainContractClient::new(&env, &contract_id);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let manufacturer_operator = Address::generate(&env);
        let pharmacy = Address::generate(&env);

        client.initialize(&admin);
        let manufacturer_id = client.register_manufacturer(
            &admin,
            &manufacturer_operator,
            &String::from_str(&env, "Uzima Pharma"),
            &String::from_str(&env, "EMA-001"),
        );
        let medication_id = client.register_medication(
            &manufacturer_operator,
            &manufacturer_id,
            &String::from_str(&env, "Biologic"),
            &String::from_str(&env, "NDC-002"),
            &true,
            &2i32,
            &8i32,
            &String::from_str(&env, "EMA"),
        );
        let batch_id = client.create_batch(
            &manufacturer_operator,
            &medication_id,
            &String::from_str(&env, "LOT-2"),
            &50u32,
            &BytesN::from_array(&env, &[4u8; 32]),
            &9_999_999u64,
        );
        let shipment_id = client.create_shipment(
            &manufacturer_operator,
            &batch_id,
            &pharmacy,
            &String::from_str(&env, "SHIP-REF-2"),
        );

        assert!(!client.log_condition_data(
            &manufacturer_operator,
            &shipment_id,
            &15i32,
            &6000u32,
            &111,
            &222
        ));
        let shipment = client.get_shipment(&shipment_id);
        assert_eq!(shipment.status, ShipmentStatus::Flagged);
        assert!(!client.run_compliance_check(&batch_id));
    }
}
