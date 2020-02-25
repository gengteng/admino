//! 加密相关工具
use crate::error::{Error, Kind};
use pbkdf2::CheckError;

/// 使用随机盐值和随机迭代次数，
/// 将密码使用 pbkdf2 算法生成 hash 并输出 Rust PBKF2 format 格式
///
/// 参考 `pbkdf2::pbkdf2_simple` 的注释
pub fn hash_pwd(pwd: &str) -> Result<String, Error> {
    pbkdf2::pbkdf2_simple(pwd, 10000).map_err(|e| Kind::CRYPTO_ERROR.with_detail(e))
}

/// 校验明文密码与 Rust PBKDF2 format 格式的 hash 密码是否一致
///
/// 参考 `pbkdf2::pbkdf2_check` 的注释
pub fn check_pwd(pwd: &str, hashed_pwd: &str) -> Result<(), Error> {
    pbkdf2::pbkdf2_check(pwd, hashed_pwd).map_err(|e| {
        match e {
            CheckError::HashMismatch => Kind::LOGIN_FAILED,
            CheckError::InvalidFormat => Kind::CRYPTO_ERROR,
        }
        .into()
    })
}
