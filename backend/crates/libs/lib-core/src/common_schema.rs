use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IDPayload {
    pub id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentifierPayload {
    pub identifier: String,
}

#[derive(Debug, Clone, Deserialize, Default, Builder)]
pub struct PageRequest {
    #[builder(default = "1")]
    pub page: u64,
    #[builder(default = "10")]
    pub page_size: u64,
}

impl PageRequest {
    pub fn max_page() -> Self {
        Self {
            page: 1,
            page_size: 10000u64,
        }
    }

    pub fn single_page(size: usize) -> Self {
        Self {
            page: 1,
            page_size: size as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageResponse<T> {
    pub total_page: u64,
    pub cur_page: u64,
    pub page_size: u64,
    pub data: Vec<T>,
}

impl<T> PageResponse<T> {
    pub fn new(total_page: u64, cur_page: u64, page_size: u64, data: Vec<T>) -> Self {
        Self {
            total_page,
            cur_page,
            page_size,
            data,
        }
    }
}

impl<T> PageResponse<T> {
    pub fn map_data_into<U: From<T>>(self) -> PageResponse<U> {
        PageResponse {
            total_page: self.total_page,
            cur_page: self.cur_page,
            page_size: self.page_size,
            data: self.data.into_iter().map(|i| i.into()).collect(),
        }
    }
}
