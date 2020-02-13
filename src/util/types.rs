use crate::error::Detail;
use phonenumber::{Mode, PhoneNumber};
use postgres_types::private::BytesMut;
use postgres_types::{FromSql, IsNull, ToSql, Type};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Phone(PhoneNumber);

impl FromStr for Phone {
    type Err = Detail;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match PhoneNumber::from_str(s) {
            Ok(number) if number.is_valid() => Ok(Self(number)),
            _ => Err(Detail::Static("invalid phone number format")),
        }
    }
}

impl Phone {
    pub fn new(src: &str) -> Result<Self, Detail> {
        Self::from_str(src)
    }

    pub fn to_e164(&self) -> String {
        self.0.format().mode(Mode::E164).to_string()
    }
}

impl fmt::Display for Phone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_e164())
    }
}

type SqlException = Box<dyn StdError + Sync + Send>;

impl<'a> FromSql<'a> for Phone {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, SqlException> {
        let s = <&str as FromSql>::from_sql(ty, raw)?;
        let pn = PhoneNumber::from_str(s)?;
        Ok(Phone(pn))
    }

    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}

impl ToSql for Phone {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, SqlException>
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

impl<'de> Deserialize<'de> for Phone {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Phone::new(&s).map_err(de::Error::custom)
    }
}
