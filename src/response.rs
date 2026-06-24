use serde::Serialize;

/// Success envelope: `{ "success": true, "data": <T> }`.
///
/// Mirrors `paidang-worker-server/src/types.ts` `apiResponse`.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}

/// Paginated payload placed inside `ApiResponse::ok`.
///
/// Serializes to `{ "list": [...], "total": N, "page": N, "pageSize": N }`,
/// matching the existing mini-program contract.
#[derive(Debug, Serialize)]
pub struct PaginatedData<T: Serialize> {
    pub list: Vec<T>,
    pub total: u64,
    pub page: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
}

impl<T: Serialize> PaginatedData<T> {
    pub fn new(list: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        Self {
            list,
            total,
            page,
            page_size,
        }
    }
}
