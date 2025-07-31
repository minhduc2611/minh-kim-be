use serde::{Deserialize, Serialize};

/// Generic paginated response structure for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// Pagination information for responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
    pub current_page: i32,
    pub total_pages: i32,
    pub has_next: bool,
    pub has_previous: bool,
}

impl PaginationInfo {
    pub fn new(total: i64, limit: i32, offset: i32) -> Self {
        let current_page = if limit > 0 { (offset / limit) + 1 } else { 1 };
        let total_pages = if limit > 0 {
            ((total as f64) / (limit as f64)).ceil() as i32
        } else {
            0
        };
        let has_next = offset + limit < total as i32;
        let has_previous = offset > 0;

        Self {
            total,
            limit,
            offset,
            current_page,
            total_pages,
            has_next,
            has_previous,
        }
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, limit: i32, offset: i32) -> Self {
        Self {
            data,
            pagination: PaginationInfo::new(total, limit, offset),
        }
    }
}

// #[derive(Deserialize)]
// pub struct PaginationQuery {
//     pub limit: Option<i32>,
//     pub offset: Option<i32>,
// }

#[derive(Deserialize)]
pub struct ListCanvasQuery {
    pub author_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}
