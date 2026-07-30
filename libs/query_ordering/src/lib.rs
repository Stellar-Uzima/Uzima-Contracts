#![no_std]

//! # Query Ordering Library
//!
//! Provides deterministic ordering primitives for search and query results
//! across Uzima contracts. Eliminates non-deterministic result ordering
//! in search queries by offering:
//!
//! - `SortField` enum with common fields (created_at, updated_at, name, id)
//! - `SortOrder` enum (Ascending, Descending)
//! - `OrderBy` struct combining field + order
//! - `sort_indices_by_key` for single-field sorting
//! - `compound_sort_indices` for multi-field sorting with tiebreakers

use soroban_sdk::{contracttype, Env};

/// Common sortable fields across Uzima contracts.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum SortField {
    CreatedAt = 0,
    UpdatedAt = 1,
    Name = 2,
    Id = 3,
}

/// Sort direction.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum SortOrder {
    Ascending = 0,
    Descending = 1,
}

/// A single ordering directive: sort by this field in this direction.
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct OrderBy {
    pub field: SortField,
    pub order: SortOrder,
}

fn insertion_sort_by_keys(env: &Env, keys: &[i128], order: SortOrder) -> soroban_sdk::Vec<u32> {
    let len = keys.len();
    let mut indices: soroban_sdk::Vec<u32> = soroban_sdk::Vec::new(env);
    for i in 0..len as u32 {
        indices.push_back(i);
    }

    for i in 1..len {
        let key_i = keys[i];
        let mut j = i;
        while j > 0 {
            let prev_val = indices.get(j as u32 - 1).unwrap();
            let prev = keys[prev_val as usize];
            let swap = match order {
                SortOrder::Ascending => prev > key_i,
                SortOrder::Descending => prev < key_i,
            };
            if !swap {
                break;
            }
            let tmp = indices.get(j as u32).unwrap();
            let prev_idx = indices.get(j as u32 - 1).unwrap();
            indices.set(j as u32, prev_idx);
            indices.set(j as u32 - 1, tmp);
            j -= 1;
        }
    }
    indices
}

/// Apply a single `OrderBy` to a slice of i128 keys.
///
/// Returns a `Vec<u32>` of original indices sorted by the requested order.
pub fn sort_indices_by_key(
    env: &Env,
    keys: &[i128],
    order: &OrderBy,
) -> soroban_sdk::Vec<u32> {
    insertion_sort_by_keys(env, keys, order.order)
}

/// Multi-field (compound) sort with a primary and tiebreaker ordering.
///
/// When two items compare equal on the primary field, the tiebreaker
/// ordering is used to break the tie. Returns sorted indices.
pub fn compound_sort_indices(
    env: &Env,
    primary_keys: &[i128],
    primary_order: &OrderBy,
    tiebreaker_keys: &[i128],
    tiebreaker_order: &OrderBy,
) -> soroban_sdk::Vec<u32> {
    let len = primary_keys.len();
    let mut indices: soroban_sdk::Vec<u32> = soroban_sdk::Vec::new(env);
    for i in 0..len as u32 {
        indices.push_back(i);
    }

    for i in 1..len {
        let key_pi = primary_keys[i];
        let key_ti = tiebreaker_keys[i];
        let mut j = i;
        while j > 0 {
            let prev_idx_val = indices.get(j as u32 - 1).unwrap() as usize;
            let key_pp = primary_keys[prev_idx_val];
            let key_tp = tiebreaker_keys[prev_idx_val];

            let primary_ord = key_pp.cmp(&key_pi);
            let dominated = match primary_ord {
                core::cmp::Ordering::Equal => {
                    let tie_ord = key_tp.cmp(&key_ti);
                    match tiebreaker_order.order {
                        SortOrder::Ascending => tie_ord == core::cmp::Ordering::Greater,
                        SortOrder::Descending => tie_ord == core::cmp::Ordering::Less,
                    }
                }
                other => match primary_order.order {
                    SortOrder::Ascending => other == core::cmp::Ordering::Greater,
                    SortOrder::Descending => other == core::cmp::Ordering::Less,
                },
            };

            if !dominated {
                break;
            }
            let tmp = indices.get(j as u32).unwrap();
            let prev_idx = indices.get(j as u32 - 1).unwrap();
            indices.set(j as u32, prev_idx);
            indices.set(j as u32 - 1, tmp);
            j -= 1;
        }
    }

    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_keys() -> [i128; 3] {
        [300, 100, 200]
    }

    #[test]
    fn test_sort_ascending() {
        let env = Env::default();
        let keys = sample_keys();
        let order = OrderBy {
            field: SortField::CreatedAt,
            order: SortOrder::Ascending,
        };
        let sorted = sort_indices_by_key(&env, &keys, &order);
        assert_eq!(sorted.get(0).unwrap(), 1);
        assert_eq!(sorted.get(1).unwrap(), 2);
        assert_eq!(sorted.get(2).unwrap(), 0);
    }

    #[test]
    fn test_sort_descending() {
        let env = Env::default();
        let keys = sample_keys();
        let order = OrderBy {
            field: SortField::CreatedAt,
            order: SortOrder::Descending,
        };
        let sorted = sort_indices_by_key(&env, &keys, &order);
        assert_eq!(sorted.get(0).unwrap(), 0);
        assert_eq!(sorted.get(1).unwrap(), 2);
        assert_eq!(sorted.get(2).unwrap(), 1);
    }

    #[test]
    fn test_compound_sort() {
        let env = Env::default();
        // Two records with same primary key (1), different tiebreaker
        let primary_keys = [1i128, 2i128, 1i128];
        let tiebreaker_keys = [200i128, 150i128, 300i128];

        let primary = OrderBy {
            field: SortField::Id,
            order: SortOrder::Ascending,
        };
        let tiebreaker = OrderBy {
            field: SortField::UpdatedAt,
            order: SortOrder::Descending,
        };

        let sorted = compound_sort_indices(
            &env, &primary_keys, &primary, &tiebreaker_keys, &tiebreaker,
        );
        // Primary ascending: id=1 first, then id=2
        // Tiebreaker descending for tied id=1: tiebreaker 300 before 200
        // So: index 2 (id=1, tb=300), index 0 (id=1, tb=200), index 1 (id=2)
        assert_eq!(sorted.get(0).unwrap(), 2);
        assert_eq!(sorted.get(1).unwrap(), 0);
        assert_eq!(sorted.get(2).unwrap(), 1);
    }

    #[test]
    fn test_empty_keys() {
        let env = Env::default();
        let keys: [i128; 0] = [];
        let order = OrderBy {
            field: SortField::Id,
            order: SortOrder::Ascending,
        };
        let sorted = sort_indices_by_key(&env, &keys, &order);
        assert_eq!(sorted.len(), 0);
    }

    #[test]
    fn test_single_element() {
        let env = Env::default();
        let keys = [42i128];
        let order = OrderBy {
            field: SortField::Id,
            order: SortOrder::Ascending,
        };
        let sorted = sort_indices_by_key(&env, &keys, &order);
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted.get(0).unwrap(), 0);
    }

    #[test]
    fn test_equal_keys_stable_order() {
        let env = Env::default();
        let keys = [5i128, 5i128, 5i128];
        let order = OrderBy {
            field: SortField::Id,
            order: SortOrder::Ascending,
        };
        let sorted = sort_indices_by_key(&env, &keys, &order);
        assert_eq!(sorted.get(0).unwrap(), 0);
        assert_eq!(sorted.get(1).unwrap(), 1);
        assert_eq!(sorted.get(2).unwrap(), 2);
    }
}
