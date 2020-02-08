use super::{IntoJsonResult, PgPool};
use crate::error::Error;
use crate::model::{Count, Id, Role};
use crate::service::role::{query_role_by_id, query_roles, query_roles_count};
use crate::util::db::Pager;
use actix_web::{web, web::Json, web::Path, Scope};

pub fn get_role_scope() -> Scope {
    web::scope("/role")
        .service(web::resource("/count").route(web::get().to(get_roles_count)))
        .service(web::resource("/list/{page}/{rows}").route(web::get().to(get_roles)))
        .service(web::resource("/{id}").route(web::get().to(get_role_by_id)))
}

async fn get_roles_count(pg_pool: web::Data<PgPool>) -> Result<Json<Count>, Error> {
    let pg_client = pg_pool.get().await?;
    query_roles_count(&pg_client).await.json()
}

async fn get_roles(
    pg_pool: web::Data<PgPool>,
    pager: Path<Pager>,
) -> Result<Json<Vec<Role>>, Error> {
    let pg_client = pg_pool.get().await?;
    query_roles(&pg_client, &pager).await.json()
}

async fn get_role_by_id(
    pg_pool: web::Data<PgPool>,
    id: web::Path<Id>,
) -> Result<Json<Role>, Error> {
    let pg_client = pg_pool.get().await?;
    query_role_by_id(&pg_client, id.into_inner()).await.json()
}
