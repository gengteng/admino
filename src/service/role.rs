use crate::error::{Error, Kind};
use crate::model::{Count, Id, Role};
use crate::util::db::Pager;
use deadpool_postgres::Client as PgClient;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn query_roles_count(pg_client: &PgClient) -> Result<Count, Error> {
    Ok(Count {
        count: pg_client
            .query_one("select count(1) from role", &[])
            .await?
            .get(0),
    })
}

pub async fn query_roles(pg_client: &PgClient, pager: &Pager) -> Result<Vec<Role>, Error> {
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

pub async fn query_role_by_id(pg_client: &PgClient, id: Id) -> Result<Role, Error> {
    if let Some(row) = pg_client
        .query_opt("select * from role where id = $1", &[&id])
        .await?
    {
        Ok(Role::from_row(row)?)
    } else {
        Err(Kind::EMPTY_RESULT.into())
    }
}
