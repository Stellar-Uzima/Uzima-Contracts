#![no_std]

//! # Pagination & Filtering Library
//!
//! Reusable pagination and filtering primitives for Soroban contract queries.
//! Provides a consistent cursor-based pagination API and composable field
//! filters across all Uzima contracts.
//!
//! ## Features
//! - Cursor-based pagination with configurable page size
//! - Composable `Filter` struct for field-level filtering
//! - Helpers for applying filters to `Vec<T>` collections
//! - Constants for page size bounds

use soroban_sdk::{contracterror, contracttype, Address, Env, String, Vec as SVec};

/// Maximum allowed page size to cap per-call CPU cost.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Default page size when none is specified.
pub const DEFAULT_PAGE_SIZE: u32 = 20;

/// Errors specific to pagination and filtering operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracterror]
pub enum PaginationError {
    /// The requested page size exceeds MAX_PAGE_SIZE.
    PageTooLarge = 1,
    /// The cursor offset exceeds the total result count.
    InvalidCursor = 2,
    /// A required filter field is missing or empty.
    MissingFilter = 3,
}

/// Cursor-based pagination request.
#[derive(Clone, Copy, Debug)]
#[contracttype]
pub struct Pagination {
    /// Index of the first item to return.
    pub offset: u64,
    /// Maximum number of items to return (capped at MAX_PAGE_SIZE).
    pub limit: u32,
}

impl Pagination {
    /// Create a first-page request.
    pub fn first(limit: u32) -> Self {
        Self {
            offset: 0,
            limit: limit.min(MAX_PAGE_SIZE),
        }
    }

    /// Create a page request from an explicit offset and limit.
    pub fn page(offset: u64, limit: u32) -> Self {
        Self {
            offset,
            limit: limit.min(MAX_PAGE_SIZE),
        }
    }

    /// Derive the next page from a response, if more items exist.
    pub fn next_from(response: &PaginationResponse, limit: u32) -> Option<Self> {
        if response.has_more {
            Some(Self::page(response.next_offset, limit))
        } else {
            None
        }
    }
}

/// Pagination result envelope.
#[derive(Clone, Debug)]
#[contracttype]
pub struct PaginationResponse {
    /// Total items in the full result set (if known).
    pub total: Option<u64>,
    /// Offset for the first item of the next page.
    pub next_offset: u64,
    /// Whether more items exist after this page.
    pub has_more: bool,
    /// Number of items returned in this page.
    pub count: u32,
}

impl PaginationResponse {
    /// Build a response from the original offset, number of returned items,
    /// and optional total count.
    pub fn from_counts(offset: u64, returned: u32, total: Option<u64>) -> Self {
        let next_offset = offset + returned as u64;
        let has_more = total
            .map(|t| next_offset < t)
            .unwrap_or(returned >= DEFAULT_PAGE_SIZE);
        Self {
            total,
            next_offset,
            has_more,
            count: returned,
        }
    }
}

/// Apply pagination to a `Vec`, returning a sub-slice and a
/// `PaginationResponse`.
pub fn paginate<T: Clone>(
    items: &SVec<T>,
    request: &Pagination,
) -> (SVec<T>, PaginationResponse) {
    let total = items.len() as u64;
    let limit = request.limit.min(MAX_PAGE_SIZE);
    let offset = request.offset.min(total) as u32;
    let end = (offset + limit).min(items.len());

    let mut page = SVec::new(items.env());
    for i in offset..end {
        page.push_back(items.get(i).unwrap());
    }

    let response = PaginationResponse::from_counts(request.offset, page.len(), Some(total));
    (page, response)
}

/// A single filter predicate applied to a query.
#[derive(Clone, Debug)]
#[contracttype]
pub enum FilterOp {
    /// Field equals value (exact match).
    Eq(String),
    /// Field contains substring.
    Contains(String),
    /// Field value is greater than threshold.
    Gt(i128),
    /// Field value is less than threshold.
    Lt(i128),
    /// Field value is greater than or equal to threshold.
    Gte(i128),
    /// Field value is less than or equal to threshold.
    Lte(i128),
}

/// A named filter to apply against a queryable field.
#[derive(Clone, Debug)]
#[contracttype]
pub struct Filter {
    /// The field name to filter on.
    pub field: String,
    /// The filter operation.
    pub op: FilterOp,
}

impl Filter {
    /// Create an equality filter.
    pub fn eq(env: &Env, field: &str, value: &str) -> Self {
        Self {
            field: String::from_str(env, field),
            op: FilterOp::Eq(String::from_str(env, value)),
        }
    }

    /// Create a "contains" filter.
    pub fn contains(env: &Env, field: &str, value: &str) -> Self {
        Self {
            field: String::from_str(env, field),
            op: FilterOp::Contains(String::from_str(env, value)),
        }
    }

    /// Create a greater-than filter.
    pub fn gt(env: &Env, field: &str, value: i128) -> Self {
        Self {
            field: String::from_str(env, field),
            op: FilterOp::Gt(value),
        }
    }

    /// Create a less-than filter.
    pub fn lt(env: &Env, field: &str, value: i128) -> Self {
        Self {
            field: String::from_str(env, field),
            op: FilterOp::Lt(value),
        }
    }
}

/// Apply a set of filters to a `Vec` of string-keyed items.
///
/// Each item is expected to be a `(String, String)` pair representing
/// `(field_name, field_value)`. Items that match all filters are retained.
pub fn apply_filters(
    items: &SVec<(String, String)>,
    filters: &SVec<Filter>,
) -> SVec<(String, String)> {
    let env = items.env();
    let mut result: SVec<(String, String)> = SVec::new(&env);

    for i in 0..items.len() {
        let item = items.get(i).unwrap();
        let mut passes = true;

        for f in 0..filters.len() {
            let filter = filters.get(f).unwrap();
            let field_val = find_field_value(items, &filter.field);

            if let Some(val) = field_val {
                if !match_filter_op(&val, &filter.op) {
                    passes = false;
                    break;
                }
            } else {
                passes = false;
                break;
            }
        }

        if passes {
            result.push_back(item);
        }
    }

    result
}

fn find_field_value(items: &SVec<(String, String)>, field: &str) -> Option<String> {
    for i in 0..items.len() {
        let item = items.get(i).unwrap();
        if item.0 == field {
            return Some(item.1.clone());
        }
    }
    None
}

fn match_filter_op(value: &String, op: &FilterOp) -> bool {
    match op {
        FilterOp::Eq(target) => value == target,
        FilterOp::Contains(sub) => {
            // Simple substring check — iterate chars
            let v_chars: SVec<u8> = value.to_buffer();
            let s_chars: SVec<u8> = sub.to_buffer();
            if s_chars.len() > v_chars.len() {
                return false;
            }
            for i in 0..=(v_chars.len() - s_chars.len()) {
                let mut found = true;
                for j in 0..s_chars.len() {
                    if v_chars.get(i + j).unwrap() != s_chars.get(j).unwrap() {
                        found = false;
                        break;
                    }
                }
                if found {
                    return true;
                }
            }
            false
        }
        FilterOp::Gt(threshold) => {
            let num = parse_i128(value);
            num > *threshold
        }
        FilterOp::Lt(threshold) => {
            let num = parse_i128(value);
            num < *threshold
        }
        FilterOp::Gte(threshold) => {
            let num = parse_i128(value);
            num >= *threshold
        }
        FilterOp::Lte(threshold) => {
            let num = parse_i128(value);
            num <= *threshold
        }
    }
}

fn parse_i128(s: &String) -> i128 {
    // Best-effort parse; returns 0 on failure (soroban has no built-in from_str for i128)
    let bytes = s.to_buffer();
    let mut result: i128 = 0;
    let mut negative = false;
    let mut started = false;

    for i in 0..bytes.len() {
        let b = bytes.get(i).unwrap();
        if b == b'-' && !started {
            negative = true;
            started = true;
        } else if b >= b'0' && b <= b'9' {
            started = true;
            result = result * 10 + (b - b'0') as i128;
        }
    }

    if negative { -result } else { result }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_pagination_first_page() {
        let env = Env::default();
        let mut items: SVec<u32> = SVec::new(&env);
        for i in 0..50u32 {
            items.push_back(i);
        }
        let req = Pagination::first(10);
        let (page, resp) = paginate(&items, &req);
        assert_eq!(page.len(), 10);
        assert!(resp.has_more);
        assert_eq!(resp.next_offset, 10);
    }

    #[test]
    fn test_pagination_last_page() {
        let env = Env::default();
        let mut items: SVec<u32> = SVec::new(&env);
        for i in 0..5u32 {
            items.push_back(i);
        }
        let req = Pagination::page(0, 10);
        let (page, resp) = paginate(&items, &req);
        assert_eq!(page.len(), 5);
        assert!(!resp.has_more);
    }

    #[test]
    fn test_pagination_next_from() {
        let env = Env::default();
        let resp = PaginationResponse::from_counts(0, 10, Some(50));
        let next = Pagination::next_from(&resp, 10);
        assert!(next.is_some());
        let next = next.unwrap();
        assert_eq!(next.offset, 10);
        assert_eq!(next.limit, 10);
    }

    #[test]
    fn test_pagination_cap_at_max() {
        let req = Pagination::first(9999);
        assert_eq!(req.limit, MAX_PAGE_SIZE);
    }

    #[test]
    fn test_filter_eq() {
        let env = Env::default();
        let f = Filter::eq(&env, "status", "active");
        assert!(match_filter_op(&String::from_str(&env, "active"), &f.op));
        assert!(!match_filter_op(&String::from_str(&env, "inactive"), &f.op));
    }

    #[test]
    fn test_filter_gt() {
        let env = Env::default();
        let f = Filter::gt(&env, "amount", 100);
        assert!(match_filter_op(&String::from_str(&env, "200"), &f.op));
        assert!(!match_filter_op(&String::from_str(&env, "50"), &f.op));
    }
}
