use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub struct Pagination {
    pub page: Option<usize>,
    pub items: Option<usize>,
    pub sorting: Option<i32>,
}

impl Pagination {
    pub fn is_fully_set(&self) -> bool {
        return self.page.is_some() && self.page.unwrap() > 0
            && self.items.is_some() && self.items.unwrap() > 0
            && self.sorting.is_some() && (self.sorting.unwrap() == 1 || self.sorting.unwrap() == -1);
    }

    pub fn is_fully_empty(&self) -> bool {
        return self.page.is_none() && self.items.is_none() && self.sorting.is_none();
    }
}



