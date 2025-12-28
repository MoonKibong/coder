//! Custom deserializers for handling HTML form data
//!
//! HTML forms send numbers as strings, so we need custom deserializers
//! that can handle both string and number formats.

use serde::{Deserialize, Deserializer};
use std::str::FromStr;

/// Deserialize a value that can be either a number or a string representation
pub fn from_str_or_number<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + Deserialize<'de>,
    T::Err: std::fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber<T> {
        String(String),
        Number(T),
    }

    match Option::<StringOrNumber<T>>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StringOrNumber::String(s)) => {
            if s.is_empty() {
                Ok(None)
            } else {
                s.parse::<T>().map(Some).map_err(serde::de::Error::custom)
            }
        }
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
    }
}

/// Deserialize f32 from either string or number
pub fn f32_from_str_or_number<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    from_str_or_number::<D, f32>(deserializer)
}

/// Deserialize i32 from either string or number
pub fn i32_from_str_or_number<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    from_str_or_number::<D, i32>(deserializer)
}

/// Deserialize bool from either string ("true"/"false") or boolean
pub fn bool_from_str_or_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        String(String),
        Bool(bool),
    }

    match Option::<StringOrBool>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StringOrBool::String(s)) => {
            match s.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Ok(Some(true)),
                "false" | "0" | "no" | "off" | "" => Ok(Some(false)),
                _ => Err(serde::de::Error::custom(format!("invalid bool string: {}", s))),
            }
        }
        Some(StringOrBool::Bool(b)) => Ok(Some(b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct TestStruct {
        #[serde(default, deserialize_with = "f32_from_str_or_number")]
        temperature: Option<f32>,
        #[serde(default, deserialize_with = "i32_from_str_or_number")]
        max_tokens: Option<i32>,
    }

    #[test]
    fn test_from_string() {
        let json = r#"{"temperature": "0.7", "max_tokens": "4096"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.temperature, Some(0.7));
        assert_eq!(result.max_tokens, Some(4096));
    }

    #[test]
    fn test_from_number() {
        let json = r#"{"temperature": 0.7, "max_tokens": 4096}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.temperature, Some(0.7));
        assert_eq!(result.max_tokens, Some(4096));
    }

    #[test]
    fn test_from_null() {
        let json = r#"{"temperature": null, "max_tokens": null}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.temperature, None);
        assert_eq!(result.max_tokens, None);
    }

    #[test]
    fn test_from_empty_string() {
        let json = r#"{"temperature": "", "max_tokens": ""}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(result.temperature, None);
        assert_eq!(result.max_tokens, None);
    }
}
