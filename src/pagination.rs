use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct Pagination {
    pub page: Option<usize>,
    pub items: Option<usize>,
    pub sorting: Option<usize>,
}

impl Pagination {
    pub fn is_fully_set(&self) -> bool {
        return self.page.is_some() && self.items.is_some() && self.sorting.is_some();
    }

    pub fn is_fully_empty(&self) -> bool {
        return self.page.is_none() && self.items.is_none() && self.sorting.is_none();
    }
}



