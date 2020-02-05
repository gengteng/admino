use actix_web::body::Body;
use actix_web::http::{header, StatusCode};
use actix_web::HttpResponse;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Display, From)]
pub enum Error {
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

impl Error {
    pub fn custom(status: StatusCode, cause: String) -> Self {
        Error::Http(HttpError { status, cause })
    }

    pub fn static_custom(status: StatusCode, cause: &str) -> Self {
        Self::custom(status, cause.to_owned())
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

#[derive(Debug, Display, Serialize)]
struct ErrorResponse {
    cause: String,
}

impl From<&Error> for ErrorResponse {
    fn from(e: &Error) -> Self {
        Self {
            cause: match e {
                Error::Text(s) => s.clone(),
                Error::Static(s) => (*s).to_string(),
                Error::Http(he) => he.cause.clone(),
                _ => format!("{}", e),
            },
        }
    }
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Http(ce) => ce.status,
            Error::Status(s) => s.to_owned(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
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
