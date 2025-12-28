//! OptionalField Pattern for Proper PATCH Updates
//!
//! Distinguishes between:
//! - Field not in request: `OptionalField::Missing` - preserve existing DB value
//! - Field in request as null: `OptionalField::Present(None)` - clear DB value
//! - Field in request with value: `OptionalField::Present(Some(value))` - set DB value

use serde::{Deserialize, Deserializer, Serialize};

/// Wrapper type that distinguishes between missing fields and null values
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use crate::utils::OptionalField;
///
/// #[derive(Deserialize)]
/// struct UpdateParams {
///     #[serde(default)]
///     pub description: OptionalField<String>,
/// }
///
/// // Missing field
/// let json = r#"{}"#;
/// let params: UpdateParams = serde_json::from_str(json).unwrap();
/// assert!(matches!(params.description, OptionalField::Missing));
///
/// // Null value
/// let json = r#"{"description": null}"#;
/// let params: UpdateParams = serde_json::from_str(json).unwrap();
/// assert!(matches!(params.description, OptionalField::Present(None)));
///
/// // Value present
/// let json = r#"{"description": "test"}"#;
/// let params: UpdateParams = serde_json::from_str(json).unwrap();
/// assert!(matches!(params.description, OptionalField::Present(Some(_))));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum OptionalField<T> {
    /// Field was not present in the request
    Missing,
    /// Field was present in the request (could be null or a value)
    Present(Option<T>),
}

impl<T> Default for OptionalField<T> {
    fn default() -> Self {
        OptionalField::Missing
    }
}

impl<'de, T> Deserialize<'de> for OptionalField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(OptionalField::Present)
    }
}

impl<T> OptionalField<T> {
    /// Returns true if the field is missing
    pub fn is_missing(&self) -> bool {
        matches!(self, OptionalField::Missing)
    }

    /// Returns true if the field is present (either null or a value)
    pub fn is_present(&self) -> bool {
        matches!(self, OptionalField::Present(_))
    }

    /// Converts to Option<Option<T>>
    pub fn as_option(&self) -> Option<&Option<T>> {
        match self {
            OptionalField::Missing => None,
            OptionalField::Present(opt) => Some(opt),
        }
    }

    /// Unwraps the Present variant, panics if Missing
    pub fn unwrap(self) -> Option<T> {
        match self {
            OptionalField::Missing => panic!("called `OptionalField::unwrap()` on a `Missing` value"),
            OptionalField::Present(opt) => opt,
        }
    }

    /// Maps the inner value if present
    pub fn map<U, F>(self, f: F) -> OptionalField<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            OptionalField::Missing => OptionalField::Missing,
            OptionalField::Present(Some(value)) => OptionalField::Present(Some(f(value))),
            OptionalField::Present(None) => OptionalField::Present(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[derive(Deserialize, Debug)]
    struct TestStruct {
        #[serde(default)]
        field: OptionalField<String>,
    }

    #[test]
    fn test_missing_field() {
        let json = r#"{}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert!(result.field.is_missing());
        assert!(!result.field.is_present());
    }

    #[test]
    fn test_null_field() {
        let json = r#"{"field": null}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert!(!result.field.is_missing());
        assert!(result.field.is_present());
        assert_eq!(result.field, OptionalField::Present(None));
    }

    #[test]
    fn test_present_field() {
        let json = r#"{"field": "value"}"#;
        let result: TestStruct = serde_json::from_str(json).unwrap();
        assert!(!result.field.is_missing());
        assert!(result.field.is_present());
        assert_eq!(
            result.field,
            OptionalField::Present(Some("value".to_string()))
        );
    }

    #[test]
    fn test_map() {
        let field = OptionalField::Present(Some(5));
        let mapped = field.map(|x| x * 2);
        assert_eq!(mapped, OptionalField::Present(Some(10)));

        let missing: OptionalField<i32> = OptionalField::Missing;
        let mapped = missing.map(|x| x * 2);
        assert_eq!(mapped, OptionalField::Missing);
    }
}
