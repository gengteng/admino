use crate::controller::IntoJsonResult;
use crate::error::Error;
use crate::model::{Count, Id, Permission, PermissionContent};
use crate::service::permission::PermissionService;
use crate::util::db::Pager;
use actix_web::web::Json;
use actix_web::{web, Scope};

pub fn get_permission_scope() -> Scope {
    web::scope("/permission")
        .service(web::resource("").route(web::post().to(create_permission)))
        .service(
            web::resource("/{id}")
                .route(web::get().to(get_permission))
                .route(web::delete().to(delete_permission))
                .route(web::patch().to(update_permission)),
        )
        .service(web::resource("/count").route(web::get().to(get_permissions_count)))
        .service(web::resource("/list/{page}/{rows}").route(web::get().to(list_permissions)))
}

async fn create_permission(
    perm_svc: web::Data<PermissionService>,
    params: Json<PermissionContent>,
) -> Result<Json<Permission>, Error> {
    perm_svc.create_permission(&params).await.json()
}

async fn get_permission(
    perm_svc: web::Data<PermissionService>,
    id: web::Path<Id>,
) -> Result<Json<Permission>, Error> {
    perm_svc.query_permission(id.into_inner()).await.json()
}

async fn delete_permission(
    perm_svc: web::Data<PermissionService>,
    id: web::Path<Id>,
) -> Result<Json<bool>, Error> {
    perm_svc.delete_permission(id.into_inner()).await.json()
}

async fn update_permission(
    perm_svc: web::Data<PermissionService>,
    id: web::Path<Id>,
    perm: web::Json<Permission>,
) -> Result<Json<bool>, Error> {
    perm_svc
        .update_permission(id.into_inner(), &perm)
        .await
        .json()
}

async fn get_permissions_count(
    perm_svc: web::Data<PermissionService>,
) -> Result<Json<Count>, Error> {
    perm_svc.query_permission_count().await.json()
}

async fn list_permissions(
    perm_svc: web::Data<PermissionService>,
    pager: web::Path<Pager>,
) -> Result<Json<Vec<Permission>>, Error> {
    perm_svc.list_permissions(&pager).await.json()
}
