//! Time slot templates DTOs.

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

/// Validate HH:MM time format.
fn valid_time_format(s: &str) -> Result<(), ValidationError> {
    let re = regex::Regex::new(r"^\d{2}:\d{2}$").map_err(|_| ValidationError::new("regex"))?;
    if re.is_match(s) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_time_format"))
    }
}

fn deserialize_optional_i8_flag<'de, D>(deserializer: D) -> Result<Option<i8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionalFlagVisitor;

    impl<'de> Visitor<'de> for OptionalFlagVisitor {
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
            deserializer.deserialize_any(FlagVisitor).map(Some)
        }
    }

    struct FlagVisitor;

    impl Visitor<'_> for FlagVisitor {
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
                    .map_err(|_| E::custom("invalid tinyint flag")),
            }
        }
    }

    deserializer.deserialize_option(OptionalFlagVisitor)
}
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateReq {
    #[validate(length(min = 1))]
    pub slot_name: String,
    #[validate(custom(function = "valid_time_format"))]
    pub start_time: String,
    #[validate(custom(function = "valid_time_format"))]
    pub end_time: String,
    pub sort_order: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_i8_flag")]
    pub is_default: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateReq {
    pub slot_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub sort_order: Option<i32>,
    #[serde(default, deserialize_with = "deserialize_optional_i8_flag")]
    pub is_default: Option<i8>,
    pub status: Option<i8>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams, ToSchema)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
#[cfg(test)]
mod tests {
    use super::{CreateReq, UpdateReq};

    #[test]
    fn create_req_accepts_boolean_is_default_from_mini_program() {
        let body = serde_json::json!({
            "slot_name": "Morning",
            "start_time": "09:00",
            "end_time": "10:00",
            "is_default": true,
            "status": 1
        });

        let req: CreateReq = serde_json::from_value(body).unwrap();

        assert_eq!(req.is_default, Some(1));
    }

    #[test]
    fn update_req_accepts_boolean_is_default_from_mini_program() {
        let body = serde_json::json!({ "is_default": false });

        let req: UpdateReq = serde_json::from_value(body).unwrap();

        assert_eq!(req.is_default, Some(0));
    }
}
