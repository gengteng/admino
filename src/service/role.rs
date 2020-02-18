use crate::error::{Error, Kind};
use crate::model::{Count, Id, Role};
use crate::service::PgPool;
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

    pub async fn query_roles(&self, pager: &Pager) -> Result<Vec<Role>, Error> {
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

    pub async fn query_role_by_id(&self, id: Id) -> Result<Role, Error> {
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
}
