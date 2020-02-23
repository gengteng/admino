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

/// 统计权限总数
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// GET /permission/count
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 11
/// content-type: application/json
/// date: Sun, 23 Feb 2020 02:04:07 GMT
///
/// {
///   "count": 2
/// }
/// ```
async fn get_permissions_count(
    perm_svc: web::Data<PermissionService>,
) -> Result<Json<Count>, Error> {
    perm_svc.query_permission_count().await.json()
}

/// 分页查询权限
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// GET /permission/list/0/2
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 97
/// content-type: application/json
/// date: Sun, 23 Feb 2020 02:05:07 GMT
///
/// [
///   {
///     "id": 1,
///     "permission_name": "权限名"
///   },
///   {
///     "id": 3,
///     "permission_name": "某某资源的删除权限"
///   }
/// ]
/// ```
async fn list_permissions(
    perm_svc: web::Data<PermissionService>,
    pager: web::Path<Pager>,
) -> Result<Json<Vec<Permission>>, Error> {
    perm_svc.list_permissions(&pager).await.json()
}

/// 创建权限
///
/// ## Example
///
/// HTTP 请求:
///
/// ```
/// POST /permission
/// Content-Type: application/json
///
/// {"permission_name": "权限名"}
/// ```
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 38
/// content-type: application/json
/// date: Sun, 23 Feb 2020 02:02:58 GMT
///
/// {
///   "id": 1,
///   "permission_name": "权限名"
/// }
/// ```
async fn create_permission(
    perm_svc: web::Data<PermissionService>,
    params: Json<PermissionContent>,
) -> Result<Json<Permission>, Error> {
    perm_svc.create_permission(&params).await.json()
}

/// 查询权限
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// GET /permission/1
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 38
/// content-type: application/json
/// date: Sun, 23 Feb 2020 02:06:13 GMT
///
/// {
///   "id": 1,
///   "permission_name": "权限名"
/// }
/// ```
async fn retrieve_permission(
    perm_svc: web::Data<PermissionService>,
    id: web::Path<Id>,
) -> Result<Json<Permission>, Error> {
    perm_svc.query_permission(id.into_inner()).await.json()
}

/// 修改权限
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// PATCH /permission/1
/// Content-Type: application/json
///
/// {"id": 1, "permission_name": "修改的权限名"}
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 0
/// content-type: text/plain; charset=utf-8
/// date: Sun, 23 Feb 2020 02:13:01 GMT
///
/// <Response body is empty>
/// ```
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

/// 删除权限
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// DELETE /permission/3
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 0
/// content-type: text/plain; charset=utf-8
/// date: Sun, 23 Feb 2020 02:13:54 GMT
///
/// <Response body is empty>
/// ```
async fn delete_permission(
    perm_svc: web::Data<PermissionService>,
    id: web::Path<Id>,
) -> Result<&'static str, Error> {
    perm_svc
        .delete_permission(id.into_inner())
        .await
        .empty_body()
}
