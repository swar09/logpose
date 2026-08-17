use crate::error::AppError;

pub const MIN_ALIAS_LENGTH: usize = 6;
pub const MAX_ALIAS_LENGTH: usize = 30;

pub const AUTO_CODE_TTL_SECONDS: u64 = 90;
pub const CUSTOM_ALIAS_TTL_SECONDS: u64 = 300;

pub const RESERVED_KEYWORDS: &[&str] = &[
    "api",
    "auth",
    "login",
    "signup",
    "billing",
    "health",
    "404",
    "dashboard",
    "settings",
    "public",
    "verify",
    "order",
    "plans",
    "webhook",
    "webhooks",
    "admin",
    "subscription",
    "subscriptions",
];

pub fn validate_custom_alias(alias: &str) -> Result<String, AppError> {
    let trimmed = alias.trim();

    if trimmed.len() < MIN_ALIAS_LENGTH || trimmed.len() > MAX_ALIAS_LENGTH {
        return Err(AppError::BadRequest(format!(
            "Custom alias must be between {} and {} characters",
            MIN_ALIAS_LENGTH, MAX_ALIAS_LENGTH
        )));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::BadRequest(
            "Custom alias can only contain alphanumeric characters, hyphens, and underscores"
                .into(),
        ));
    }

    let lower = trimmed.to_lowercase();
    if RESERVED_KEYWORDS.contains(&lower.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Custom alias '{trimmed}' is a reserved keyword"
        )));
    }

    Ok(trimmed.to_string())
}

pub fn is_custom_alias(short_code: &str) -> bool {
    short_code.len() >= MIN_ALIAS_LENGTH
}

pub fn get_cache_ttl_for_code(short_code: &str) -> u64 {
    if is_custom_alias(short_code) {
        CUSTOM_ALIAS_TTL_SECONDS
    } else {
        AUTO_CODE_TTL_SECONDS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_custom_aliases() {
        let max_len_alias = "a".repeat(30);
        let valid_aliases = [
            "my-url",
            "my_url",
            "summer2026",
            "product-launch-2026",
            "sale_page_v1",
            "abcdef",
            "123456",
            max_len_alias.as_str(),
        ];

        for alias in valid_aliases {
            let result = validate_custom_alias(alias);
            assert!(result.is_ok(), "Expected valid alias: {alias}");
            assert_eq!(result.unwrap(), alias.trim());
        }
    }

    #[test]
    fn test_alias_too_short() {
        let short_aliases = ["", "a", "ab", "abc", "abcd", "abcde", "12345"];
        for alias in short_aliases {
            let result = validate_custom_alias(alias);
            assert!(result.is_err(), "Expected alias too short: {alias}");
        }
    }

    #[test]
    fn test_alias_too_long() {
        let long_alias = "a".repeat(31);
        let result = validate_custom_alias(&long_alias);
        assert!(result.is_err(), "Expected alias too long: {long_alias}");
    }

    #[test]
    fn test_invalid_characters() {
        let invalid_aliases = [
            "my url",
            "my.url.com",
            "my@alias",
            "alias#1",
            "link$dollar",
            "hello!",
            "slash/in/alias",
            "colon:test",
            "plus+alias",
        ];

        for alias in invalid_aliases {
            let result = validate_custom_alias(alias);
            assert!(result.is_err(), "Expected invalid character: {alias}");
        }
    }

    #[test]
    fn test_reserved_keywords() {
        let reserved = [
            "api",
            "auth",
            "login",
            "signup",
            "billing",
            "health",
            "404",
            "dashboard",
            "settings",
            "public",
            "verify",
            "admin",
            "API",
            "Auth",
            "LOGIN",
        ];

        for keyword in reserved {
            let result = validate_custom_alias(keyword);
            assert!(
                result.is_err(),
                "Expected reserved keyword error: {keyword}"
            );
        }
    }

    #[test]
    fn test_is_custom_alias_partition() {
        assert!(!is_custom_alias("a"));
        assert!(!is_custom_alias("ab"));
        assert!(!is_custom_alias("abc"));
        assert!(!is_custom_alias("Z19a"));
        assert!(!is_custom_alias("12345"));

        assert!(is_custom_alias("123456"));
        assert!(is_custom_alias("my-url"));
        assert!(is_custom_alias("custom-alias-example"));
    }

    #[test]
    fn test_cache_ttl_policy() {
        assert_eq!(get_cache_ttl_for_code("Z19a"), 90);
        assert_eq!(get_cache_ttl_for_code("12345"), 90);
        assert_eq!(get_cache_ttl_for_code("custom-alias"), 300);
        assert_eq!(get_cache_ttl_for_code("summer_sale_2026"), 300);
    }
}
