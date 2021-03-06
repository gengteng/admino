//! 各种工具类型
use crate::error::{Error, Exception, Kind};
use phonenumber::{Mode, PhoneNumber};
use postgres_types::private::BytesMut;
use postgres_types::{FromSql, IsNull, ToSql, Type};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// 为类型 $t 实现 Serialize
macro_rules! impl_se {
    ($t:ty) => {
        impl Serialize for $t {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }
    };
}

/// 为类型 $t 实现 Deserialize
macro_rules! impl_de {
    ($t:ty) => {
        impl<'de> Deserialize<'de> for $t {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                <$t>::from_str(&s).map_err(de::Error::custom)
            }
        }
    };
}

macro_rules! sql_str_val {
    ($t:ty, $n:expr) => {
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                self.0.fmt(f)
            }
        }

        impl<'a> FromSql<'a> for $t {
            fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Exception> {
                let s = <&str as FromSql>::from_sql(ty, raw)?;
                let phone =
                    <$t>::from_str(s).map_err(|_| concat!("从数据库中读出的", $n, "格式错误"))?;
                Ok(phone)
            }

            fn accepts(ty: &Type) -> bool {
                <&str as FromSql>::accepts(ty)
            }
        }

        impl ToSql for $t {
            fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Exception>
            where
                Self: Sized,
            {
                self.0.to_sql(ty, out)
            }

            fn accepts(ty: &Type) -> bool
            where
                Self: Sized,
            {
                <&str as ToSql>::accepts(ty)
            }

            to_sql_checked!();
        }
    };
}

/// 6位数字验证码
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct AuthCode {
    pub code: String,
}

impl Distribution<AuthCode> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AuthCode {
        AuthCode {
            code: format!("{:06}", rng.gen_range(0u32, 1_000_000)),
        }
    }
}

impl Default for AuthCode {
    fn default() -> Self {
        rand::random()
    }
}

impl AuthCode {
    pub fn new(s: &str) -> Result<Self, Error> {
        if s.len() != 6 || !s.bytes().all(|c| c.is_ascii_digit()) {
            return Err(Kind::INVALID_AUTH_CODE.into());
        }

        Ok(Self { code: s.into() })
    }
}

impl fmt::Display for AuthCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.code.fmt(f)
    }
}

/// 手机号
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Phone(PhoneNumber);

impl FromStr for Phone {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match PhoneNumber::from_str(s) {
            Ok(number) if number.is_valid() => Ok(Self(number)),
            _ => Err(Kind::INVALID_PHONE_NUMBER.into()),
        }
    }
}

impl Phone {
    pub fn new(src: &str) -> Result<Self, Error> {
        Self::from_str(src)
    }
    /// 转换为 E.164 格式的字符串
    pub fn to_e164(&self) -> String {
        self.0.format().mode(Mode::E164).to_string()
    }
}

impl fmt::Display for Phone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.to_e164().fmt(f)
    }
}

impl<'a> FromSql<'a> for Phone {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Exception> {
        let s = <&str as FromSql>::from_sql(ty, raw)?;
        let phone =
            Phone::from_str(s).map_err(|_| String::from("从数据库中读出的手机号格式错误"))?;
        Ok(phone)
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}

impl ToSql for Phone {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Exception>
    where
        Self: Sized,
    {
        let s = self.to_e164();
        s.to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool
    where
        Self: Sized,
    {
        <&str as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl Serialize for Phone {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_e164())
    }
}

impl_de!(Phone);

/// 电子邮箱
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Email(String);

impl FromStr for Email {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if mailchecker::is_valid(s) {
            Ok(Self(s.into()))
        } else {
            Err(Kind::INVALID_EMAIL.into())
        }
    }
}

impl Email {
    pub fn new(src: &str) -> Result<Self, Error> {
        Self::from_str(src)
    }
}

sql_str_val!(Email, "电子邮箱");
impl_se!(Email);
impl_de!(Email);

/// 用户名
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Username(String);

impl FromStr for Username {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.chars().count();
        if len < 6 || len > 32 {
            Err(Kind::INVALID_USERNAME.into())
        } else {
            Ok(Self(s.into()))
        }
    }
}

impl Username {
    pub fn new(s: &str) -> Result<Self, Error> {
        Self::from_str(s)
    }
}

sql_str_val!(Username, "用户名");
impl_se!(Username);
impl_de!(Username);
