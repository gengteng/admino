use actix_web::{Scope, web, web::Json};
use crate::model::Role;
use crate::error::Error;
use crate::util::db::QueryCondition;
use crate::service::role::query_roles;
use super::{PgPool, IntoJsonResult};

pub fn get_role_scope() -> Scope {
    web::scope("/role")
        .service(web::resource("/list").route(web::get().to(get_roles)))
}

async fn get_roles(pg_pool: web::Data<PgPool>, condition: Json<QueryCondition>) -> Result<Json<Vec<Role>>, Error> {
    let pg_client = pg_pool.get().await?;
    query_roles(&pg_client, condition).await.json()
}