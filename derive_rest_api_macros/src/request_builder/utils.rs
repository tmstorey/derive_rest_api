//! Request builder specific utility functions.
//!
//! This module contains utilities that are specific to the RequestBuilder
//! derive macro and not generally useful for other macros.

/// Extract path parameters from a URL path template.
///
/// # Examples
///
/// - "/api/users/{id}/posts/{post_id}" -> ["id", "post_id"]
/// - "/api/users" -> []
/// - "/api/{version}/users/{id}" -> ["version", "id"]
pub(super) fn extract_path_params(path: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut param = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '}' {
                    chars.next(); // consume '}'
                    break;
                }
                param.push(chars.next().unwrap());
            }
            if !param.is_empty() {
                params.push(param);
            }
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_params() {
        assert_eq!(
            extract_path_params("/api/users/{id}/posts/{post_id}"),
            vec!["id", "post_id"]
        );
        assert_eq!(extract_path_params("/api/users"), Vec::<String>::new());
        assert_eq!(
            extract_path_params("/api/{version}/users/{id}"),
            vec!["version", "id"]
        );
    }
}
