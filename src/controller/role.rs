//! 角色相关控制器
//!
use super::IntoJsonResult;
use crate::controller::EmptyBody;
use crate::error::Error;
use crate::model::{Count, Id, Role, RoleContent};
use crate::service::role::RoleService;
use crate::util::db::Pager;
use actix_web::{web, web::Json, web::Path, Scope};

pub fn get_role_scope() -> Scope {
    web::scope("/role")
        .service(web::resource("").route(web::post().to(create_role)))
        .service(web::resource("/count").route(web::get().to(get_roles_count)))
        .service(web::resource("/list/{page}/{rows}").route(web::get().to(list_roles)))
        .service(
            web::resource("/{id}")
                .route(web::get().to(retrieve_role))
                .route(web::patch().to(update_role))
                .route(web::delete().to(delete_role)),
        )
}

/// 统计角色总数
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// GET /role/count
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 11
/// content-type: application/json
/// date: Sat, 22 Feb 2020 12:59:37 GMT
///
/// {
///   "count": 2
/// }
/// ```
async fn get_roles_count(role_svc: web::Data<RoleService>) -> Result<Json<Count>, Error> {
    role_svc.query_roles_count().await.json()
}

async fn list_roles(
    role_svc: web::Data<RoleService>,
    pager: Path<Pager>,
) -> Result<Json<Vec<Role>>, Error> {
    role_svc.list_roles(&pager).await.json()
}

/// 创建角色
///
/// ## Example
///
/// HTTP 请求:
///
/// ```
/// POST /role
/// Content-Type: application/json
///
/// {"name": "角色名1", "max_user": 100, "max_permission": 200}
/// ```
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 64
/// content-type: application/json
/// date: Fri, 21 Feb 2020 16:39:05 GMT
///
/// {
///   "id": 6,
///   "name": "角色名1",
///   "max_user": 100,
///   "max_permission": 200
/// }
/// ```
async fn create_role(
    role_svc: web::Data<RoleService>,
    params: Json<RoleContent>,
) -> Result<Json<Role>, Error> {
    role_svc.create_role(&params).await.json()
}

/// 查询角色
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// GET /role/16
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 65
/// content-type: application/json
/// date: Sat, 22 Feb 2020 13:11:04 GMT
///
/// {
///   "id": 16,
///   "name": "角色名2",
///   "max_user": 100,
///   "max_permission": 200
/// }
/// ```
async fn retrieve_role(
    role_svc: web::Data<RoleService>,
    id: web::Path<Id>,
) -> Result<Json<Role>, Error> {
    role_svc.query_role(id.into_inner()).await.json()
}

/// 修改角色
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// PATCH http://localhost:30000/role/6
/// Content-Type: application/json
///
/// {"id": 6, "name": "角色名1", "max_user": 100, "max_permission": 200}
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 0
/// content-type: text/plain; charset=utf-8
/// date: Sat, 22 Feb 2020 12:32:15 GMT
///
/// <Response body is empty>
/// ```
async fn update_role(
    role_svc: web::Data<RoleService>,
    id: web::Path<Id>,
    role: web::Json<Role>,
) -> Result<&'static str, Error> {
    role_svc
        .update_role(id.into_inner(), &role)
        .await
        .empty_body()
}

/// 删除角色
///
/// ## Example
///
/// HTTP 请求:
/// ```
/// DELETE http://localhost:30000/role/6
/// ```
///
/// HTTP 响应:
/// ```
/// HTTP/1.1 200 OK
/// content-length: 0
/// content-type: text/plain; charset=utf-8
/// date: Sat, 22 Feb 2020 12:32:15 GMT
///
/// <Response body is empty>
/// ```
async fn delete_role(
    role_svc: web::Data<RoleService>,
    id: web::Path<Id>,
) -> Result<&'static str, Error> {
    role_svc.delete_role(id.into_inner()).await.empty_body()
}
