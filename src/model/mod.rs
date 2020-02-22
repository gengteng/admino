//! 数据模型定义
pub use serde::{Deserialize, Serialize};
pub use tokio_pg_mapper_derive::PostgresMapper;

mod permission;
mod role;
mod user;

pub use permission::*;
pub use role::*;
pub use user::*;

pub type Id = i64;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Count {
    pub count: i64,
}

impl From<i64> for Count {
    fn from(count: i64) -> Self {
        Self { count }
    }
}
