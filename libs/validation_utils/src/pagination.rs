#![no_std]
//! pagination - Cursor-based pagination for large contract query paths.
//!
//! Provides a consistent pagination API across all Uzima contracts that
//! return collections (records, audit entries, provider lists, etc.).

use soroban_sdk::{contracttype, Vec as SVec};

/// Maximum page size to cap per-call CPU cost.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Default page size when none is specified.
pub const DEFAULT_PAGE_SIZE: u32 = 20;

/// Pagination parameters for a query.
#[derive(Clone, Copy, Debug)]
#[contracttype]
pub struct PageRequest {
    /// Offset-based cursor: index of first item to return.
    pub offset: u64,
    /// Number of items to return (capped at MAX_PAGE_SIZE).
    pub limit: u32,
}

impl PageRequest {
    pub fn first(limit: u32) -> Self {
        Self { offset: 0, limit: limit.min(MAX_PAGE_SIZE) }
    }

    pub fn next_from(current: &PageResponse, limit: u32) -> Option<Self> {
        if current.has_more {
            Some(Self { offset: current.next_offset, limit })
        } else {
            None
        }
    }
}

/// Pagination result envelope.
#[derive(Clone)]
#[contracttype]
pub struct PageResponse {
    /// Total items in the result set (if known).
    pub total: Option<u64>,
    /// Offset for the next page.
    pub next_offset: u64,
    /// Whether more items exist after this page.
    pub has_more: bool,
    /// Number of items returned in this page.
    pub count: u32,
}

impl PageResponse {
    pub fn from_slice(offset: u64, returned: u32, total: Option<u64>) -> Self {
        let next_offset = offset + returned as u64;
        let has_more = total.map(|t| next_offset < t).unwrap_or(returned >= DEFAULT_PAGE_SIZE);
        Self { total, next_offset, has_more, count: returned }
    }
}

/// Apply pagination to a Vec, returning a sub-slice and a PageResponse.
pub fn paginate<T: Clone>(
    items: &SVec<T>,
    request: &PageRequest,
) -> (SVec<T>, PageResponse) {
    let total = items.len() as u64;
    let limit = request.limit.min(MAX_PAGE_SIZE);
    let offset = request.offset.min(total) as u32;
    let end = (offset + limit).min(items.len());

    let mut page = SVec::new(items.env());
    for i in offset..end {
        page.push_back(items.get(i).unwrap());
    }

    let response = PageResponse::from_slice(request.offset, page.len(), Some(total));
    (page, response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_first_page() {
        let env = Env::default();
        let mut items: SVec<u32> = SVec::new(&env);
        for i in 0..50u32 { items.push_back(i); }
        let req = PageRequest::first(10);
        let (page, resp) = paginate(&items, &req);
        assert_eq!(page.len(), 10);
        assert!(resp.has_more);
        assert_eq!(resp.next_offset, 10);
    }

    #[test]
    fn test_last_page_no_more() {
        let env = Env::default();
        let mut items: SVec<u32> = SVec::new(&env);
        for i in 0..5u32 { items.push_back(i); }
        let req = PageRequest { offset: 0, limit: 10 };
        let (page, resp) = paginate(&items, &req);
        assert_eq!(page.len(), 5);
        assert!(!resp.has_more);
    }

    #[test]
    fn test_cap_at_max_page_size() {
        let req = PageRequest::first(9999);
        assert_eq!(req.limit, MAX_PAGE_SIZE);
    }
}
