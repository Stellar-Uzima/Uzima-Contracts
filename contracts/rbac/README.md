# Role-Based Access Control (RBAC) Contract

A Soroban smart contract for standardized role and permission management across the Uzima healthcare ecosystem.

## Quick Start

### Features

- 🔐 **Centralized Role Management** - Single contract for all role assignments
- 👥 **Multiple Roles** - 8 predefined healthcare roles (Admin, Doctor, Patient, etc.)
- ✅ **Admin-Only Control** - Only admins can assign/remove roles
- 🔍 **Flexible Queries** - Check roles, get members, get statistics
- 📊 **Role Statistics** - Count members, list all addresses with role
- 🎯 **Convenience Functions** - Quick checks (is_doctor, is_patient, etc.)
- 📝 **Audit Trail** - Complete history of role assignments

### Core Functions

| Function | Purpose | Auth |
|----------|---------|------|
| `assign_role()` | Assign role to address | Admin |
| `remove_role()` | Remove role from address | Admin |
| `has_role()` | Check if address has role | Public |
| `get_roles()` | Get all roles for address | Public |
| `has_any_role()` | Check for any of multiple roles | Public |
| `has_all_roles()` | Check for all of multiple roles | Public |
| `is_doctor()` | Quick check if doctor | Public |
| `is_patient()` | Quick check if patient | Public |
| `is_admin()` | Quick check if admin | Public |
| `get_role_members()` | Get all addresses with role | Public |
| `get_role_member_count()` | Count addresses with role | Public |

### Roles

- **Admin** (0) - Full system access
- **Doctor** (1) - Healthcare provider
- **Patient** (2) - Patient/subject
- **Staff** (3) - Healthcare staff
- **Insurer** (4) - Insurance provider
- **Researcher** (5) - Researcher
- **Auditor** (6) - Compliance officer
- **Service** (7) - System service

### Initialization

```rust
let config = RBACConfig {
    emit_events: true,
    max_roles_per_address: 10,
};

RBAC::initialize(env, admin_address, config);
```

### Assign & Remove Roles

```rust
// Assign role (admin only)
RBAC::assign_role(env, user_address, Role::Doctor);

// Remove role (admin only)
RBAC::remove_role(env, user_address, Role::Doctor);
```

### Check & Query Roles

```rust
// Check single role
let is_doc = RBAC::has_role(env, address, Role::Doctor);

// Get all roles
let roles = RBAC::get_roles(env, address);

// Check multiple roles
let has_permissions = RBAC::has_any_role(env, address, roles_vec);

// Convenience check
let is_doc = RBAC::is_doctor(env, address);

// Get all doctors
let doctors = RBAC::get_role_members(env, Role::Doctor);

// Count doctors
let count = RBAC::get_role_member_count(env, Role::Doctor);
```

## Module Structure

- **`lib.rs`** - Main contract with 16 public functions
- **`types.rs`** - Role enums and data structures
- **`storage.rs`** - Storage operations and persistence
- **`queries.rs`** - Query logic for role information
- **`test.rs`** - 22 comprehensive unit tests

## Building

```bash
cargo build -p rbac --target wasm32-unknown-unknown --release
```

## Testing

```bash
cargo test -p rbac --lib
```

## Authorization Flow

```
User calls assign_role()
    ↓
Admin requires authorization
    ↓
Admin proves identity cryptographically
    ↓
Role added to storage
    ↓
Assignment history recorded
    ↓
Event emitted (if enabled)
    ↓
Return success
```

## Integration Example

```rust
// In another contract
use rbac::RBAC;
use rbac::types::Role;

#[contract]
pub struct MedicalRecords;

#[contractimpl]
impl MedicalRecords {
    pub fn read_patient_record(
        env: Env,
        patient: Address,
        reader: Address,
    ) -> PatientRecord {
        // Verify reader is authorized
        if !RBAC::is_doctor(env.clone(), reader.clone()) {
            panic!("Only doctors can read records");
        }
        
        // Proceed with access
        Self::get_record(env, patient)
    }
}
```

## Key Design Decisions

1. **Centralized**: All roles managed in one contract
2. **Immutable History**: Assignment records never deleted
3. **Admin-Only**: Only admins can change roles
4. **Dual Indexing**: Indexed by address AND by role for fast lookups
5. **Flexible Queries**: Multiple ways to check permissions
6. **No Deletion**: Roles removed but history persists for audit

## Storage Structure

```
AddressRoles(Address) -> Vec<Role>
RoleMembers(Role) -> Vec<Address>
Assignment(u64) -> RoleAssignment record
Admin -> Address
Config -> RBACConfig
```

## Compliance

✅ Access control enforcement  
✅ Audit trail for compliance  
✅ Authorization proofs  
✅ Segregation of duties

## See Also

- Full documentation: [docs/RBAC.md](../docs/RBAC.md)
- Health data logging: [../health_data_access_logging/](../health_data_access_logging/)
- Medical records: [../medical_records/](../medical_records/)
