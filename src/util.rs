//! Shared authorization guards.
//!
//! A valid JWT represents a logged-in provider. The database `user.role` column
//! is retained for compatibility but is never used for authorization.

use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Require that the logged-in provider owns the photographer-scoped resource.
pub fn require_owner(auth: &AuthUser, photographer_id: i32) -> Result<(), AppError> {
    if auth.user_id == photographer_id {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "forbidden photographer resource".into(),
        ))
    }
}
use serde::Deserializer;
use serde::de::{self, Visitor};
use std::fmt;

pub fn deserialize_optional_i8<'de, D>(deserializer: D) -> Result<Option<i8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionalI8Visitor;

    impl<'de> Visitor<'de> for OptionalI8Visitor {
        type Value = Option<i8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean, tinyint, numeric string, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(I8Visitor).map(Some)
        }
    }

    struct I8Visitor;

    impl Visitor<'_> for I8Visitor {
        type Value = i8;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean or tinyint")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(if value { 1 } else { 0 })
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            i8::try_from(value).map_err(|_| E::custom("tinyint out of range"))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            i8::try_from(value).map_err(|_| E::custom("tinyint out of range"))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                "true" => Ok(1),
                "false" => Ok(0),
                _ => value
                    .parse::<i8>()
                    .map_err(|_| E::custom("invalid tinyint")),
            }
        }
    }

    deserializer.deserialize_option(OptionalI8Visitor)
}
