use derive_rest_api_macros::RequestBuilder;

// Validation function for testing
fn validate_positive(value: &u32) -> Result<(), String> {
    if *value > 0 {
        Ok(())
    } else {
        Err("value must be positive".to_string())
    }
}

// Validation function for email
fn validate_email(value: &String) -> Result<(), String> {
    if value.contains('@') {
        Ok(())
    } else {
        Err("invalid email format".to_string())
    }
}

// Validation function for string length
fn validate_min_length(value: &String) -> Result<(), String> {
    if value.len() >= 3 {
        Ok(())
    } else {
        Err("must be at least 3 characters".to_string())
    }
}

#[test]
fn test_validation_success() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct CreateUser {
        #[request_builder(validate = "validate_email")]
        email: String,
        age: u32,
    }

    let result = CreateUserBuilder::new()
        .email("user@example.com".to_string())
        .age(25)
        .build();

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, "user@example.com");
    assert_eq!(user.age, 25);
}

#[test]
fn test_validation_failure() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct CreateUser {
        #[request_builder(validate = "validate_email")]
        email: String,
        age: u32,
    }

    let result = CreateUserBuilder::new()
        .email("invalid-email".to_string())
        .age(25)
        .build();

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Validation failed"));
    assert!(err.contains("email"));
    assert!(err.contains("invalid email format"));
}

#[test]
fn test_validation_multiple_fields() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct CreatePost {
        #[request_builder(validate = "validate_min_length")]
        title: String,
        #[request_builder(validate = "validate_min_length")]
        content: String,
    }

    // Both valid
    let result1 = CreatePostBuilder::new()
        .title("Hello".to_string())
        .content("World".to_string())
        .build();
    assert!(result1.is_ok());

    // First field invalid
    let result2 = CreatePostBuilder::new()
        .title("Hi".to_string())
        .content("World".to_string())
        .build();
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("title"));

    // Second field invalid
    let result3 = CreatePostBuilder::new()
        .title("Hello".to_string())
        .content("Hi".to_string())
        .build();
    assert!(result3.is_err());
    assert!(result3.unwrap_err().to_string().contains("content"));
}

#[test]
fn test_validation_optional_field_some() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct UpdateUser {
        id: u64,
        #[request_builder(validate = "validate_email")]
        email: Option<String>,
    }

    // Valid email
    let result1 = UpdateUserBuilder::new()
        .id(123)
        .email("valid@example.com".to_string())
        .build();
    assert!(result1.is_ok());

    // Invalid email
    let result2 = UpdateUserBuilder::new()
        .id(123)
        .email("invalid".to_string())
        .build();
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("email"));
}

#[test]
fn test_validation_optional_field_none() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct UpdateUser {
        id: u64,
        #[request_builder(validate = "validate_email")]
        email: Option<String>,
    }

    // Optional field not set - validation should not run
    let result = UpdateUserBuilder::new()
        .id(123)
        .build();
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.id, 123);
    assert_eq!(user.email, None);
}

#[test]
fn test_validation_with_default() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct Config {
        #[request_builder(default, validate = "validate_positive")]
        timeout: u32,
    }

    // Not set, uses default (0), but validation should run and fail
    let result = ConfigBuilder::new()
        .build();

    // Since timeout defaults to 0, validation should fail
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timeout"));
}

#[test]
fn test_validation_with_default_valid() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct Config {
        #[request_builder(default, validate = "validate_positive")]
        timeout: u32,
    }

    // Set to valid value
    let result = ConfigBuilder::new()
        .timeout(30)
        .build();

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.timeout, 30);
}

#[test]
fn test_validation_combined_with_into() {
    #[derive(RequestBuilder, Debug, PartialEq)]
    struct CreateUser {
        #[request_builder(into, validate = "validate_email")]
        email: String,
    }

    // Valid email with Into
    let result1 = CreateUserBuilder::new()
        .email("user@example.com".to_string())
        .build();
    assert!(result1.is_ok());

    // Invalid email with Into
    let result2 = CreateUserBuilder::new()
        .email("invalid".to_string())
        .build();
    assert!(result2.is_err());
}
