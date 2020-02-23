//! 所有错误类型定义
use actix_web::body::Body;
use actix_web::http::{header, StatusCode};
use actix_web::HttpResponse;
use serde::Serialize;
use std::error::Error as StdError;
use std::fmt;

/// 动态错误类型，可从各种错误类型转换而来
pub type Exception = Box<dyn StdError + Sync + Send + 'static>;

/// 包括了静态错误信息和运行时动态错误信息的错误类型
#[derive(Debug)]
pub struct Error {
    kind: &'static Kind,
    detail: Option<Exception>,
}

impl Error {
    /// 简单构造一个只包含静态错误信息的错误对象
    pub fn simple(kind: &'static Kind) -> Self {
        Self::from(kind)
    }

    /// 构造包含静态/动态错误信息的错误对象
    pub fn new<E: StdError + Sync + Send + 'static>(kind: &'static Kind, error: E) -> Self {
        Self {
            kind,
            detail: Some(Box::new(error)),
        }
    }

    /// 获取静态错误信息
    pub fn kind(&self) -> &'static Kind {
        self.kind
    }

    /// 获取动态错误信息
    pub fn detail(&self) -> Option<&Exception> {
        self.detail.as_ref()
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        if let Some(e) = e.source() {
            if let Some(e) = e.downcast_ref::<tokio_postgres::error::DbError>() {
                if let Some(constraint) = e.constraint() {
                    // 违反了 unique 约束，表示是这个错误： DUPLICATE_VALUE
                    if constraint.find("unique").is_some() {
                        return Kind::DUPLICATE_VALUE.into();
                    }
                }
            }
        }

        Error::new(Kind::DB_ERROR, e)
    }
}

/// 为 Error 实现 From<$t>
///
/// $t 为错误类型， $k 为对应的 Kind 值
///
macro_rules! simple_to_error {
    ($t:ty, $k:expr) => {
        impl From<$t> for Error {
            fn from(e: $t) -> Self {
                Error::new($k, e)
            }
        }
    };
}

simple_to_error!(
    deadpool::managed::PoolError<tokio_postgres::error::Error>,
    Kind::DB_POOL_ERROR
);
simple_to_error!(tokio_pg_mapper::Error, Kind::DB_MAPPER_ERROR);
simple_to_error!(
    deadpool::managed::PoolError<redis::RedisError>,
    Kind::CACHE_POOL_ERROR
);
simple_to_error!(redis::RedisError, Kind::CACHE_ERROR);

/// 静态错误类型
///
/// # 错误码规则
/// 1. 0 表示成功；
/// 2. 正数表示客户端错误，返回 HTTP 响应码为 4XX；
/// 3. 负数表示服务端错误，返回 HTTP 响应码为 5XX。
///
#[derive(Debug)]
pub struct Kind {
    code: i64,
    message: &'static str,
    status: StatusCode,
}

impl From<&'static Kind> for Error {
    fn from(kind: &'static Kind) -> Self {
        Self { kind, detail: None }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}(HTTP{}): {}", self.code, self.status, self.message)
    }
}

#[allow(dead_code)]
impl Kind {
    /// 与一个动态错误信息共同构造一个错误对象
    pub fn with_detail<E: StdError + Sync + Send + 'static>(&'static self, error: E) -> Error {
        Error {
            kind: self,
            detail: Some(Box::new(error)),
        }
    }

    /// 返回静态的错误码
    pub fn code(&self) -> i64 {
        self.code
    }

    /// 返回对应的HTTP状态码
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// 返回静态错误描述
    pub fn message(&self) -> &'static str {
        self.message
    }

    /// 只在此处使用的构造函数
    const fn new(code: i64, message: &'static str, status: StatusCode) -> Self {
        Self {
            code,
            message,
            status,
        }
    }

    /// 成功
    pub const OK: &'static Kind = &Kind::new(0, "成功", StatusCode::OK);

    /// 用户尚未登录(1)
    pub const USER_NOT_SIGNED_IN: &'static Kind =
        &Kind::new(1, "用户尚未登录", StatusCode::UNAUTHORIZED);
    /// 用户没有权限(2)
    pub const NO_PERMISSION: &'static Kind =
        &Kind::new(2, "用户没有权限", StatusCode::UNAUTHORIZED);
    /// 用户名格式错误(3)
    pub const INVALID_USERNAME: &'static Kind =
        &Kind::new(3, "用户名格式错误", StatusCode::BAD_REQUEST);
    /// 手机号格式错误(4)
    pub const INVALID_PHONE_NUMBER: &'static Kind =
        &Kind::new(4, "手机号格式错误", StatusCode::BAD_REQUEST);
    /// 电子邮件格式错误(5)
    pub const INVALID_EMAIL: &'static Kind =
        &Kind::new(5, "电子邮件格式错误", StatusCode::BAD_REQUEST);
    /// 登录失败(6)
    pub const LOGIN_FAILED: &'static Kind = &Kind::new(6, "登录失败", StatusCode::UNAUTHORIZED);
    /// 验证码错误(7)
    pub const INVALID_AUTH_CODE: &'static Kind =
        &Kind::new(7, "验证码错误", StatusCode::BAD_REQUEST);
    /// 违反唯一性约束(8)
    pub const DUPLICATE_VALUE: &'static Kind =
        &Kind::new(8, "违反唯一性约束", StatusCode::BAD_REQUEST);
    /// 请求的资源不存在(9)
    pub const EMPTY_RESULT: &'static Kind =
        &Kind::new(8, "请求的资源不存在", StatusCode::NOT_FOUND);

    /// 未知服务器错误(-1)
    pub const UNKNOWN: &'static Kind =
        &Kind::new(-1, "未知服务器错误", StatusCode::INTERNAL_SERVER_ERROR);
    /// 内部数据格式错误(-2)
    pub const DATA_FORMAT: &'static Kind =
        &Kind::new(-2, "内部数据格式错误", StatusCode::INTERNAL_SERVER_ERROR);
    /// 数据库错误(-3)
    pub const DB_ERROR: &'static Kind =
        &Kind::new(-3, "数据库错误", StatusCode::INTERNAL_SERVER_ERROR);
    /// 数据库连接池错误(-4)
    pub const DB_POOL_ERROR: &'static Kind =
        &Kind::new(-4, "数据库连接池错误", StatusCode::INTERNAL_SERVER_ERROR);
    /// 数据格式错误(-5)
    pub const DB_MAPPER_ERROR: &'static Kind =
        &Kind::new(-5, "数据格式错误", StatusCode::INTERNAL_SERVER_ERROR);
    /// 缓存错误(-6)
    pub const CACHE_ERROR: &'static Kind =
        &Kind::new(-6, "缓存错误", StatusCode::INTERNAL_SERVER_ERROR);
    /// 缓存连接池错误(-7)
    pub const CACHE_POOL_ERROR: &'static Kind =
        &Kind::new(-7, "缓存连接池错误", StatusCode::INTERNAL_SERVER_ERROR);
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if let Some(detail) = &self.detail {
            write!(f, "{}, {}", self.kind, detail)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

/// Error 转换为 HTTP 响应报文体的格式，用于序列化为 JSON
///
/// # Example
///
/// 客户端错误：
/// ```
/// HTTP/1.1 400 Bad Request
/// content-length: 44
/// content-type: application/json; charset=utf-8
/// date: Sat, 22 Feb 2020 06:40:13 GMT
///
/// {
///   "code": 8,
///   "message": "违反唯一性约束"
/// }
/// ```
///
/// 服务端错误:
/// ```
/// HTTP/1.1 500 Internal Server Error
/// content-length: 171
/// content-type: application/json; charset=utf-8
/// date: Sat, 22 Feb 2020 06:42:21 GMT
///
/// {
///   "code": -7,
///   "message": "缓存连接池错误",
///   "detail": "An error occured while creating a new object: 由于目标计算机积极拒绝，无法连接。 (os error 10061)"
/// }
/// ```
///
#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: i64,
    message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

impl From<&Error> for ErrorResponse {
    fn from(e: &Error) -> Self {
        Self {
            code: e.kind.code,
            message: e.kind.message.into(),
            detail: e.detail().map(|e| format!("{}", e)),
        }
    }
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        self.kind.status
    }

    fn error_response(&self) -> HttpResponse<Body> {
        let err_resp = ErrorResponse::from(self);
        let body = match serde_json::to_string(&err_resp) {
            Ok(body) => body,
            Err(e) => {
                return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR).set_body(Body::from(
                    format!("Failed to serialize the error response to JSON: {}", e),
                ))
            }
        };

        let mut resp = HttpResponse::new(self.status_code());
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json; charset=utf-8"),
        );
        resp.set_body(Body::from(body))
    }
}
