use super::*;

/// 权限
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "permission")]
pub struct Permission {
    pub id: Id,
    pub permission_name: String,
}

/// 角色权限
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PostgresMapper)]
#[pg_mapper(table = "role_permission")]
pub struct RolePermission {
    pub role_id: Id,
    pub permission_id: Id,
}
