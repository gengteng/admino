use deadpool_postgres::Client as PgClient;
use actix_web::web;
use crate::error::Error;
use crate::util::db::QueryCondition;
use crate::model::Role;
use tokio_pg_mapper::FromTokioPostgresRow;
use itertools::Itertools;

pub async fn query_roles(pg_client: &PgClient, condition: web::Json<QueryCondition>) -> Result<Vec<Role>, Error> {
    let condition = condition.into_inner();

    let sql = {
        let mut sql = String::from("select * from roles limit $1 offset $2");
        if let Some(order_by) = condition.order_by {
            sql += " order by ";
            sql += &order_by.iter().join(",");
        }
        sql
    };

    let rows = pg_client
        .query(sql.as_str(),
                    &[&condition.pager.limit(), &condition.pager.offset()])
        .await?;

    let mut roles = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        roles.push(Role::from_row_ref(row)?);
    }

    Ok(roles)
}