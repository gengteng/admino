//! 加密相关工具
use crate::error::{Error, Kind};
use actix_web::error::BlockingError;
use rand::{thread_rng, Rng};

/// 使用随机盐值和随机迭代次数，
/// 将密码使用 pbkdf2 算法生成 hash 并输出 Rust PBKF2 format 格式
///
/// 参考 `pbkdf2::pbkdf2_simple` 的注释
pub async fn hash_pwd(pwd: String) -> Result<String, Error> {
    Ok(actix_rt::blocking::run(move || {
        let mut rng = thread_rng();
        let iter_count: u32 = rng.gen_range(100, 1000);
        pbkdf2::pbkdf2_simple(&pwd, iter_count)
    })
    .await
    .map_err(|e| match e {
        BlockingError::Error(e) => Kind::CRYPTO_ERROR.with_detail(e),
        BlockingError::Canceled => Kind::WORKER_THREAD_ERROR.into(),
    })?)
}

/// 校验明文密码与 Rust PBKF2 format 格式的 hash 密码是否一致
///
/// 参考 `pbkdf2::pbkdf2_check` 的注释
pub async fn check_pwd(pwd: String, hashed_pwd: String) -> Result<(), Error> {
    Ok(
        actix_rt::blocking::run(move || pbkdf2::pbkdf2_check(&pwd, &hashed_pwd))
            .await
            .map_err(|e| match e {
                BlockingError::Error(e) => Kind::CRYPTO_ERROR.with_detail(e),
                BlockingError::Canceled => Kind::WORKER_THREAD_ERROR.into(),
            })?,
    )
}
