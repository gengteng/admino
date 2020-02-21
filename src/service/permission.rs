use crate::error::{Error, Kind};
use crate::model::{Count, Id, Permission, PermissionContent};
use crate::opt::PgPool;
use crate::util::db::Pager;
use tokio_pg_mapper::FromTokioPostgresRow;

pub struct PermissionService {
    pg_pool: PgPool,
}

impl PermissionService {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }

    pub async fn query_permission_count(&self) -> Result<Count, Error> {
        let pg_client = self.pg_pool.get().await?;

        Ok(Count {
            count: pg_client
                .query_one("select count(1) from permission", &[])
                .await?
                .get(0),
        })
    }

    pub async fn list_permissions(&self, pager: &Pager) -> Result<Vec<Permission>, Error> {
        let pg_client = self.pg_pool.get().await?;

        let rows = pg_client
            .query(
                "select * from permission limit $1 offset $2",
                &[&pager.limit(), &pager.offset()],
            )
            .await?;

        let mut permissions = Vec::with_capacity(rows.len());

        for row in rows.iter() {
            permissions.push(Permission::from_row_ref(row)?);
        }

        Ok(permissions)
    }

    pub async fn query_permission(&self, id: Id) -> Result<Permission, Error> {
        let pg_client = self.pg_pool.get().await?;

        if let Some(row) = pg_client
            .query_opt("select * from permission where id = $1", &[&id])
            .await?
        {
            Ok(Permission::from_row(row)?)
        } else {
            Err(Kind::EMPTY_RESULT.into())
        }
    }

    pub async fn create_permission(&self, params: &PermissionContent) -> Result<Permission, Error> {
        let pg_client = self.pg_pool.get().await?;

        let row = pg_client
            .query_one(
                "insert into permission(permission_name) values($1) returning *",
                &[&params.permission_name],
            )
            .await?;

        Ok(Permission::from_row(row)?)
    }

    pub async fn delete_permission(&self, id: Id) -> Result<bool, Error> {
        let pg_client = self.pg_pool.get().await?;

        let count = pg_client
            .execute("delete from permission where id = $1", &[&id])
            .await?;

        Ok(count == 1)
    }

    pub async fn update_permission(&self, id: Id, permission: &Permission) -> Result<bool, Error> {
        let pg_client = self.pg_pool.get().await?;

        let count = pg_client
            .execute(
                "update permission set id = $1, permission_name = $2 where id = $3",
                &[&permission.id, &permission.permission_name, &id],
            )
            .await?;

        Ok(count == 1)
    }
}
