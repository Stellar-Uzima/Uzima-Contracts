# Role-Based Access Control (RBAC) Contract

## Overview

The Role-Based Access Control (RBAC) Contract (#298) is a Soroban smart contract that provides a standardized, reusable system for managing roles and permissions across the Uzima healthcare ecosystem. This contract enforces access control by restricting functions and resources based on assigned roles.

## Problem Statement

Healthcare systems require sophisticated access control mechanisms:
- **No Standardization**: Different contracts implement their own permission systems
- **Code Duplication**: RBAC logic repeated across multiple contracts
- **Maintenance Burden**: Changes to permission logic require updates in many places
- **Permission Conflicts**: Inconsistent role definitions across contracts
- **Scalability Issues**: Hard to add new roles or permissions to existing contracts

## Solution

The RBAC Contract provides:

✅ **Centralized Role Management** - Single contract for all role assignments  
✅ **Reusable Permissions** - Other contracts can query this contract  
✅ **Admin Control** - Only admins can assign/remove roles  
✅ **Authorization Enforcement** - Built-in require_auth() for all operations  
✅ **Flexible Role System** - Multiple roles, supports multiple roles per address  
✅ **Complete Traceability** - Assignment history with timestamps and admin records  

## Roles

The contract defines 8 healthcare roles:

| Role | Code | Purpose |
|------|------|---------|
| **Admin** | 0 | Full system access, manages RBAC |
| **Doctor** | 1 | Healthcare provider, can access patient records |
| **Patient** | 2 | Patient/subject of healthcare |
| **Staff** | 3 | Healthcare staff (nurse, technician, etc.) |
| **Insurer** | 4 | Insurance provider, handles billing/claims |
| **Researcher** | 5 | Researcher accessing de-identified data |
| **Auditor** | 6 | Compliance and audit officer |
| **Service** | 7 | System service account for automated processes |

## Core Functions

### Role Assignment & Management

#### 1. `assign_role(address, role)` - Admin Only
Assigns a role to an address.

**Arguments:**
- `address`: Address to assign role to
- `role`: Role to assign

**Returns:** 
- `true` if role assigned successfully
- `false` if address already has role or max roles exceeded

**Requires:** Admin authorization

**Events:** ROLE/ASSIGN (if events enabled in config)

```rust
let success = RBAC::assign_role(
    env,
    user_address,
    Role::Doctor,
);
```

#### 2. `remove_role(address, role)` - Admin Only
Removes a role from an address.

**Arguments:**
- `address`: Address to remove role from
- `role`: Role to remove

**Returns:**
- `true` if role removed successfully
- `false` if address didn't have that role

**Requires:** Admin authorization

**Events:** ROLE/REMOVE (if events enabled in config)

```rust
let success = RBAC::remove_role(
    env,
    user_address,
    Role::Doctor,
);
```

### Role Queries

#### 3. `has_role(address, role)` - Public
Checks if an address has a specific role.

**Arguments:**
- `address`: Address to check
- `role`: Role to check for

**Returns:** `true` if address has the role

```rust
if RBAC::has_role(env, doctor_address, Role::Doctor) {
    // Allow medical record access
}
```

#### 4. `get_roles(address)` - Public
Gets all roles assigned to an address.

**Arguments:**
- `address`: Address to get roles for

**Returns:** Vector of all roles

```rust
let roles = RBAC::get_roles(env, user_address);
```

#### 5. `has_any_role(address, roles)` - Public
Checks if address has any of the specified roles.

**Returns:** `true` if address has at least one role from the list

**Use Case:** "Is user a doctor OR researcher?"

```rust
let mut required_roles = Vec::with_capacity(&env, 2);
required_roles.push_back(Role::Doctor);
required_roles.push_back(Role::Researcher);

if RBAC::has_any_role(env, address, required_roles) {
    // User is either doctor or researcher
}
```

#### 6. `has_all_roles(address, roles)` - Public
Checks if address has all specified roles.

**Returns:** `true` if address has all roles in the list

**Use Case:** "Is user both doctor AND auditor?"

```rust
let mut required_roles = Vec::with_capacity(&env, 2);
required_roles.push_back(Role::Doctor);
required_roles.push_back(Role::Auditor);

if RBAC::has_all_roles(env, address, required_roles) {
    // User is both doctor and auditor
}
```

### Role Information

#### 7. `get_address_roles(address)` - Public
Gets detailed role information for an address.

**Returns:** AddressRoles struct with:
- Address
- All assigned roles
- Role count

```rust
let role_info = RBAC::get_address_roles(env, user_address);
println!("Roles: {:?}", role_info.roles);
println!("Count: {}", role_info.role_count);
```

#### 8. `get_role_members(role)` - Public
Gets all addresses with a specific role.

**Arguments:**
- `role`: Role to query

**Returns:** Vector of all addresses with that role

**Use Case:** Finding all doctors in the system

```rust
let doctors = RBAC::get_role_members(env, Role::Doctor);
```

#### 9. `get_role_member_count(role)` - Public
Gets count of addresses with a specific role.

**Returns:** Number of addresses with the role

**Use Case:** Statistics on role distribution

```rust
let doctor_count = RBAC::get_role_member_count(env, Role::Doctor);
```

### Convenience Functions

#### 10. `is_admin(address)` - Public
Checks if address is an admin.

```rust
if RBAC::is_admin(env, address) { /* ... */ }
```

#### 11. `is_doctor(address)` - Public
Checks if address is a doctor.

```rust
if RBAC::is_doctor(env, address) { /* ... */ }
```

#### 12. `is_patient(address)` - Public
Checks if address is a patient.

```rust
if RBAC::is_patient(env, address) { /* ... */ }
```

#### 13. `is_staff(address)` - Public
Checks if address is staff.

```rust
if RBAC::is_staff(env, address) { /* ... */ }
```

### Configuration

#### 14. `initialize(admin, config)` - One-time Setup
Initializes the RBAC contract.

**Arguments:**
- `admin`: Admin address with full permissions
- `config`: RBAC configuration

**Panics:** If already initialized

```rust
let config = RBACConfig {
    emit_events: true,
    max_roles_per_address: 10,
};

RBAC::initialize(env, admin_address, config);
```

#### 15. `update_config(config)` - Admin Only
Updates RBAC configuration.

```rust
let new_config = RBACConfig {
    emit_events: true,
    max_roles_per_address: 5,
};

RBAC::update_config(env, new_config);
```

#### 16. `get_config()` - Public
Gets current RBAC configuration.

```rust
let config = RBAC::get_config(env);
```

## Data Structures

### Role Enum
```rust
pub enum Role {
    Admin = 0,
    Doctor = 1,
    Patient = 2,
    Staff = 3,
    Insurer = 4,
    Researcher = 5,
    Auditor = 6,
    Service = 7,
}
```

### AddressRoles
```rust
pub struct AddressRoles {
    pub address: Address,           // The address
    pub roles: Vec<Role>,           // All assigned roles
    pub role_count: u32,            // Number of roles
}
```

### RoleAssignment
```rust
pub struct RoleAssignment {
    pub address: Address,           // Who got the role
    pub role: Role,                 // Which role
    pub assigned_at: u64,           // Timestamp
    pub assigned_by: Address,       // Which admin assigned it
}
```

### RBACConfig
```rust
pub struct RBACConfig {
    pub emit_events: bool,                    // Event emission
    pub max_roles_per_address: u32,           // Max roles per address
}
```

## Acceptance Criteria ✅

- [x] **Only admins can assign roles** - Admin authorization required via require_auth()
- [x] **Only admins can remove roles** - Admin authorization required
- [x] **Unauthorized actions are blocked** - require_auth() enforces authorization
- [x] **Roles persist correctly** - Persistent storage ensures data durability
- [x] **assign_role(address, role)** - Implemented with admin-only access
- [x] **remove_role(address, role)** - Implemented with admin-only access
- [x] **has_role(address, role)** - Implemented with public access

## Authorization Model

- **Admin**: Can assign/remove roles and update configuration
- **Any Address**: Can query roles (public read access)

**Example Authorization Flow:**
```
1. Admin calls assign_role()
2. Admin addresses require_auth() -> Admin proves identity
3. Role assigned to target address
4. Event emitted (optional)
5. History recorded

Later:
1. Any address queries has_role()
2. No auth required (read-only)
3. Boolean returned
```

## Storage Optimization

The contract uses efficient storage patterns:

1. **Persistent Storage**: All role data in persistent storage
2. **Indexed by Address**: Fast role lookup by address
3. **Indexed by Role**: Fast member lookup by role
4. **Assignment History**: Complete audit trail with IDs

## Events

The contract emits events for:

1. **INIT/RBAC**: When contract is initialized
2. **ROLE/ASSIGN**: When a role is assigned (if events enabled)
3. **ROLE/REMOVE**: When a role is removed (if events enabled)
4. **CONFIG/UPDATE**: When configuration is updated

## Security Considerations

1. **Authorization**: All admin actions require require_auth()
2. **No Self-Service**: Only admins can modify roles
3. **Audit Trail**: All assignments recorded with admin and timestamp
4. **No Deletion**: Roles can be removed but history persists
5. **Immutable History**: Assignment records cannot be modified

## Compliance

✅ **Access Control** - Enforces role-based permissions  
✅ **Audit Trail** - Complete history of role changes  
✅ **Authorization** - Cryptographic proof requirements  
✅ **Segregation of Duties** - Only admins manage roles

## Integration with Other Contracts

Other contracts can integrate with RBAC:

```rust
// In another contract
use rbac::RBAC;
use rbac::types::Role;

#[contract]
pub struct PatientRecords;

#[contractimpl]
impl PatientRecords {
    pub fn read_record(env: Env, patient: Address, reader: Address) {
        // Check if reader is a doctor
        if !RBAC::is_doctor(env.clone(), reader.clone()) {
            panic!("Only doctors can read records");
        }
        
        // Check if reader is authorized for this patient
        if !RBAC::has_role(env.clone(), reader.clone(), Role::Doctor) {
            panic!("Unauthorized");
        }
        
        // Proceed with reading patient record
    }
}
```

## Usage Examples

### Setup
```rust
let admin = Address::random(&env);
let config = RBACConfig {
    emit_events: true,
    max_roles_per_address: 10,
};

RBAC::initialize(env, admin, config);
```

### Assign Multiple Roles
```rust
RBAC::assign_role(env.clone(), doctor_addr, Role::Doctor);
RBAC::assign_role(env.clone(), doctor_addr, Role::Researcher);
```

### Check Permissions
```rust
if RBAC::is_doctor(env, user_addr) {
    // Allow medical actions
}
```

### Get All Doctors
```rust
let doctors = RBAC::get_role_members(env, Role::Doctor);
```

### Revoke Permissions
```rust
RBAC::remove_role(env, user_addr, Role::Doctor);
```

## Testing

The contract includes 22 comprehensive tests covering:

- ✅ Initialization and contract lifecycle
- ✅ Single and multiple role assignments
- ✅ Role removal and non-existent role handling
- ✅ Role queries (has_role, get_roles)
- ✅ Multiple role combinations (has_any, has_all)
- ✅ Role member queries
- ✅ Convenience functions (is_doctor, is_patient, etc.)
- ✅ Configuration management
- ✅ Role limits enforcement
- ✅ All role types
- ✅ Complex multi-operation scenarios

Run tests with:
```bash
cargo test -p rbac --lib
```

## Future Enhancements

1. **Role Hierarchies**: Doctor > Staff hierarchy
2. **Temporal Roles**: Time-limited role assignments
3. **Scope-Based Roles**: Roles scoped to specific patients
4. **Delegation**: Admins can delegate authority
5. **Multi-Signature**: Require multiple admins for sensitive operations
6. **Role Templates**: Pre-defined role bundles
7. **Batch Operations**: Assign roles to multiple addresses

## Files Overview

```
contracts/rbac/
├── Cargo.toml                 # Package manifest
└── src/
    ├── lib.rs                 # Main contract (16 public functions)
    ├── types.rs               # Role enums and data structures
    ├── storage.rs             # Storage operations
    ├── queries.rs             # Query functions
    └── test.rs                # 22 comprehensive tests
```

## Contribution

To extend this contract:

1. Add new roles to the `Role` enum in `types.rs`
2. Add new query functions in `queries.rs`
3. Add storage helpers in `storage.rs`
4. Expose new contract methods in `lib.rs`
5. Add tests in `test.rs`

## License

MIT License - See repository for details
