pub use serde::{Deserialize, Serialize};
pub use tokio_pg_mapper_derive::PostgresMapper;

mod permission;
mod role;
mod user;

pub use permission::*;
pub use role::*;
pub use user::*;

pub type Id = i64;
