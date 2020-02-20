use crate::error::{Error, Kind};
use crate::model::{Count, Id, Role, RoleContent};
use crate::opt::PgPool;
use crate::util::db::Pager;
use tokio_pg_mapper::FromTokioPostgresRow;

pub struct RoleService {
    pg_pool: PgPool,
}

impl RoleService {
    pub fn new(pg_pool: PgPool) -> Self {
        Self { pg_pool }
    }

    pub async fn query_roles_count(&self) -> Result<Count, Error> {
        let pg_client = self.pg_pool.get().await?;

        Ok(Count {
            count: pg_client
                .query_one("select count(1) from role", &[])
                .await?
                .get(0),
        })
    }

    pub async fn list_roles(&self, pager: &Pager) -> Result<Vec<Role>, Error> {
        let pg_client = self.pg_pool.get().await?;

        let rows = pg_client
            .query(
                "select * from role limit $1 offset $2",
                &[&pager.limit(), &pager.offset()],
            )
            .await?;

        let mut roles = Vec::with_capacity(rows.len());

        for row in rows.iter() {
            roles.push(Role::from_row_ref(row)?);
        }

        Ok(roles)
    }

    pub async fn query_role(&self, id: Id) -> Result<Role, Error> {
        let pg_client = self.pg_pool.get().await?;

        if let Some(row) = pg_client
            .query_opt("select * from role where id = $1", &[&id])
            .await?
        {
            Ok(Role::from_row(row)?)
        } else {
            Err(Kind::EMPTY_RESULT.into())
        }
    }

    pub async fn create_role(&self, params: &RoleContent) -> Result<Role, Error> {
        let pg_client = self.pg_pool.get().await?;

        let row = pg_client
            .query_one(
                "insert into role(name, max_user, max_permission) values($1, $2, $3) returning *",
                &[&params.name, &params.max_user, &params.max_permission],
            )
            .await?;

        Ok(Role::from_row(row)?)
    }

    pub async fn delete_role(&self, id: Id) -> Result<bool, Error> {
        let pg_client = self.pg_pool.get().await?;

        let count = pg_client
            .execute("delete from role where id = $1", &[&id])
            .await?;

        Ok(count == 1)
    }

    pub async fn update_role(&self, id: Id, role: &RoleContent) -> Result<bool, Error> {
        let pg_client = self.pg_pool.get().await?;

        let count = pg_client
            .execute(
                "update role set name = $1, max_user = $2, max_permission = $3 where id = $4",
                &[&role.name, &role.max_user, &role.max_permission, &id],
            )
            .await?;

        Ok(count == 1)
    }
}
