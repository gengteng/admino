//! 权限相关控制器
//!
use crate::controller::{EmptyBody, IntoJsonResult};
use crate::error::Error;
use crate::model::{Count, Id, Permission, PermissionContent};
use crate::service::permission::PermissionService;
use crate::util::db::Pager;
use actix_web::web::Json;
use actix_web::{web, Scope};

/// 获取所有权限相关的所有路由
pub fn get_permission_scope() -> Scope {
    web::scope("/permission")
        .service(web::resource("").route(web::post().to(create_permission)))
        .service(web::resource("/count").route(web::get().to(get_permissions_count)))
        .service(web::resource("/list/{page}/{rows}").route(web::get().to(list_permissions)))
        .service(
            web::resource("/{id}")
                .route(web::get().to(retrieve_permission))
                .route(web::patch().to(update_permission))
                .route(web::delete().to(delete_permission)),
        )
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

async fn create_permission(
    perm_svc: web::Data<PermissionService>,
    params: Json<PermissionContent>,
) -> Result<Json<Permission>, Error> {
    perm_svc.create_permission(&params).await.json()
}

async fn retrieve_permission(
    perm_svc: web::Data<PermissionService>,
    id: web::Path<Id>,
) -> Result<Json<Permission>, Error> {
    perm_svc.query_permission(id.into_inner()).await.json()
}

async fn delete_permission(
    perm_svc: web::Data<PermissionService>,
    id: web::Path<Id>,
) -> Result<&'static str, Error> {
    perm_svc
        .delete_permission(id.into_inner())
        .await
        .empty_body()
}

async fn update_permission(
    perm_svc: web::Data<PermissionService>,
    id: web::Path<Id>,
    perm: web::Json<Permission>,
) -> Result<&'static str, Error> {
    perm_svc
        .update_permission(id.into_inner(), &perm)
        .await
        .empty_body()
}
