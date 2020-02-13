use actix_web::body::Body;
use actix_web::http::{header, StatusCode};
use actix_web::HttpResponse;
use serde::Serialize;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Display, From)]
pub enum Detail {
    Io(std::io::Error),
    Borrow(std::cell::BorrowError),
    BorrowMut(std::cell::BorrowMutError),
    ParseInt(std::num::ParseIntError),

    Actix(actix_web::error::Error),
    Serde(serde_json::error::Error),

    Postgres(tokio_postgres::error::Error),
    PgPool(deadpool::managed::PoolError<tokio_postgres::error::Error>),
    PgMap(tokio_pg_mapper::Error),

    Redis(redis::RedisError),
    RedisPool(deadpool::managed::PoolError<redis::RedisError>),

    Failure(failure::Error),

    Static(&'static str),
    Text(String),
    Status(StatusCode),
    Http(HttpError),
}

impl StdError for Detail {}

#[derive(Debug)]
pub struct Error {
    kind: &'static Kind,
    detail: Option<Detail>,
}

impl Error {
    pub fn kind(kind: &'static Kind) -> Self {
        Self::from(kind)
    }

    pub fn new(kind: &'static Kind, detail: Detail) -> Self {
        Self {
            kind,
            detail: Some(detail),
        }
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(e: tokio_postgres::Error) -> Self {
        Error::new(Kind::DB_ERROR, Detail::from(e))
    }
}

impl From<deadpool::managed::PoolError<tokio_postgres::error::Error>> for Error {
    fn from(e: deadpool::managed::PoolError<tokio_postgres::error::Error>) -> Self {
        Error {
            kind: Kind::DB_POOL_ERROR,
            detail: Some(Detail::from(e)),
        }
    }
}

impl From<tokio_pg_mapper::Error> for Error {
    fn from(e: tokio_pg_mapper::Error) -> Self {
        Error {
            kind: Kind::DB_MAPPER_ERROR,
            detail: Some(Detail::from(e)),
        }
    }
}

impl From<deadpool::managed::PoolError<redis::RedisError>> for Error {
    fn from(e: deadpool::managed::PoolError<redis::RedisError>) -> Self {
        Error {
            kind: Kind::CACHE_POOL_ERROR,
            detail: Some(Detail::from(e)),
        }
    }
}

impl From<redis::RedisError> for Error {
    fn from(e: redis::RedisError) -> Self {
        Error {
            kind: Kind::CACHE_ERROR,
            detail: Some(Detail::from(e)),
        }
    }
}

/// 错误
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
    pub fn with_detail(&'static self, detail: Detail) -> Error {
        Error {
            kind: self,
            detail: Some(detail),
        }
    }

    pub fn code(&self) -> i64 {
        self.code
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    const fn new(code: i64, message: &'static str, status: StatusCode) -> Self {
        Self {
            code,
            message,
            status,
        }
    }

    /// 成功
    pub const OK: &'static Kind = &Kind::new(0, "成功", StatusCode::OK);

    /// 失败（客户端错误， code>0 & status=4XX）
    pub const USER_NOT_SIGNED_IN: &'static Kind =
        &Kind::new(1, "用户尚未登录", StatusCode::UNAUTHORIZED);
    pub const NO_PERMISSION: &'static Kind =
        &Kind::new(2, "用户没有权限", StatusCode::UNAUTHORIZED);
    pub const INVALID_PHONE_NUMBER: &'static Kind =
        &Kind::new(3, "手机号错误", StatusCode::BAD_REQUEST);
    pub const INVALID_USERNAME_PASSWORD: &'static Kind =
        &Kind::new(4, "用户名/密码错误", StatusCode::BAD_REQUEST);
    pub const INVALID_AUTH_CODE: &'static Kind =
        &Kind::new(5, "验证码错误", StatusCode::BAD_REQUEST);
    pub const DUPLICATE_IDENTITY: &'static Kind =
        &Kind::new(6, "该身份标识已经注册", StatusCode::BAD_REQUEST);

    /// 错误（服务端错误，code<0 & status=5XX)
    pub const UNKNOWN: &'static Kind =
        &Kind::new(-1, "未知服务器错误", StatusCode::INTERNAL_SERVER_ERROR);
    pub const DATA_FORMAT: &'static Kind =
        &Kind::new(-2, "内部数据格式错误", StatusCode::INTERNAL_SERVER_ERROR);
    pub const DB_ERROR: &'static Kind =
        &Kind::new(-3, "数据库错误", StatusCode::INTERNAL_SERVER_ERROR);
    pub const DB_POOL_ERROR: &'static Kind =
        &Kind::new(-4, "数据库连接池错误", StatusCode::INTERNAL_SERVER_ERROR);
    pub const DB_MAPPER_ERROR: &'static Kind =
        &Kind::new(-5, "数据格式错误", StatusCode::INTERNAL_SERVER_ERROR);
    pub const CACHE_ERROR: &'static Kind =
        &Kind::new(-6, "缓存错误", StatusCode::INTERNAL_SERVER_ERROR);
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

#[derive(Debug)]
pub struct HttpError {
    status: StatusCode,
    cause: String,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.cause)
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: i64,
    message: String,
    detail: String,
}

impl From<&Error> for ErrorResponse {
    fn from(e: &Error) -> Self {
        Self {
            code: e.kind.code,
            message: e.kind.message.into(),
            detail: if let Some(detail) = &e.detail {
                format!("{}", detail)
            } else {
                "".into()
            },
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
