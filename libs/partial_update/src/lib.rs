#![no_std]

use soroban_sdk::{contracterror, contracttype, Bytes, Env, String, Vec};

/// Errors that can occur during partial-update operations.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PartialUpdateError {
    /// The requested field was not found in the update set.
    FieldNotFound = 1,
    /// A nested path segment was invalid.
    InvalidPath = 2,
    /// The update list is empty — nothing to apply.
    EmptyUpdate = 3,
    /// The record to merge into is empty or missing.
    RecordNotFound = 4,
}

/// A single field update: the dot-separated path to the field and the new value.
///
/// For simple (non-nested) updates, `path` is just the field name.
/// For nested updates, use dot notation: `"address.city"`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldUpdate {
    pub path: String,
    pub value: Bytes,
}

/// A reusable container for a list of selective field updates.
///
/// Use the builder pattern to construct an update set, then call
/// [`PartialUpdate::merge_with_existing`] to apply it to an on-chain record.
///
/// # Example (conceptual — requires an Env)
/// ```ignore
/// let update = PartialUpdate::new(&env)
///     .set(&env, "name", value_bytes_1)
///     .set(&env, "address.city", value_bytes_2)
///     .build();
/// ```
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartialUpdate {
    pub updates: Vec<FieldUpdate>,
}

impl PartialUpdate {
    /// Create a new builder with an empty update list.
    pub fn new(env: &Env) -> Self {
        Self {
            updates: Vec::new(env),
        }
    }

    /// Append a field update to the builder.
    ///
    /// * `path` — Dot-separated field path (e.g. `"email"` or `"address.zip"`).
    /// * `value` — The new value encoded as bytes.
    pub fn set(self, env: &Env, path: &str, value: &Bytes) -> Self {
        let mut updates = self.updates;
        updates.push_back(FieldUpdate {
            path: String::from_str(env, path),
            value: value.clone(),
        });
        Self { updates }
    }

    /// Finalize the builder and return the `PartialUpdate`.
    ///
    /// Returns [`PartialUpdateError::EmptyUpdate`] if no fields were added.
    pub fn build(self, env: &Env) -> Result<Self, PartialUpdateError> {
        if self.updates.is_empty() {
            return Err(PartialUpdateError::EmptyUpdate);
        }
        Ok(self)
    }

    /// Apply this partial update to an existing record stored as a `Vec<FieldUpdate>`.
    ///
    /// For each field in `self.updates`, the function searches `existing` for a
    /// matching `path` and replaces its value. If the path does not exist in the
    /// existing record it is appended (additive merge).
    ///
    /// Returns the merged record as a new `Vec<FieldUpdate>`.
    pub fn merge_with_existing(
        self,
        env: &Env,
        existing: &Vec<FieldUpdate>,
    ) -> Result<Vec<FieldUpdate>, PartialUpdateError> {
        let merged = Vec::new(env);

        // Copy existing entries into a mutable working list
        let mut working = Vec::new(env);
        for entry in existing.iter() {
            working.push_back(entry);
        }

        // Apply each update
        for update in self.updates.iter() {
            let mut replaced = false;
            let mut idx: u32 = 0;
            let len = working.len();

            while idx < len {
                let entry = working.get(idx).unwrap();
                if entry.path == update.path {
                    // Replace value in place
                    working.set(idx, FieldUpdate {
                        path: update.path.clone(),
                        value: update.value.clone(),
                    });
                    replaced = true;
                    break;
                }
                idx += 1;
            }

            if !replaced {
                // Additive: path didn't exist, append it
                working.push_back(FieldUpdate {
                    path: update.path.clone(),
                    value: update.value.clone(),
                });
            }
        }

        Ok(working)
    }

    /// Check whether a specific field path is present in the update set.
    pub fn has_field(&self, env: &Env, path: &str) -> bool {
        let target = String::from_str(env, path);
        for update in self.updates.iter() {
            if update.path == target {
                return true;
            }
        }
        false
    }

    /// Return the number of pending field updates.
    pub fn len(&self) -> u32 {
        self.updates.len()
    }

    /// Return `true` if the update set is empty.
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }
}

#[cfg(test)]
mod tests;
