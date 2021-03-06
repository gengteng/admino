//! 数据库相关工具
use serde::{Deserialize, Serialize};

/// 分页查询条件
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pager {
    pub rows: i64,
    pub page: i64,
}

impl Pager {
    pub fn limit(&self) -> i64 {
        self.rows
    }

    pub fn offset(&self) -> i64 {
        self.rows * self.page
    }
}

/// 所有查询条件
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryCondition {
    pub pager: Pager,
    pub order_by: Option<Vec<String>>,
}
