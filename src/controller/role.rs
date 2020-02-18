use super::IntoJsonResult;
use crate::error::Error;
use crate::model::{Count, Id, Role};
use crate::service::role::RoleService;
use crate::util::db::Pager;
use actix_web::{web, web::Json, web::Path, Scope};

pub fn get_role_scope() -> Scope {
    web::scope("/role")
        .service(web::resource("/count").route(web::get().to(get_roles_count)))
        .service(web::resource("/list/{page}/{rows}").route(web::get().to(get_roles)))
        .service(web::resource("/{id}").route(web::get().to(get_role_by_id)))
}

async fn get_roles_count(role_svc: web::Data<RoleService>) -> Result<Json<Count>, Error> {
    role_svc.query_roles_count().await.json()
}

async fn get_roles(
    role_svc: web::Data<RoleService>,
    pager: Path<Pager>,
) -> Result<Json<Vec<Role>>, Error> {
    role_svc.query_roles(&pager).await.json()
}

async fn get_role_by_id(
    role_svc: web::Data<RoleService>,
    id: web::Path<Id>,
) -> Result<Json<Role>, Error> {
    role_svc.query_role_by_id(id.into_inner()).await.json()
}
