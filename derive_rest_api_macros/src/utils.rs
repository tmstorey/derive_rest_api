//! Shared utility functions for procedural macros.
//!
//! This module contains common utilities that can be used across different
//! derive macros in this crate.

use syn;

/// Extract the inner type from an Option<T> type.
///
/// Returns Some(&T) if the type is Option<T>, None otherwise.
/// Handles various Option type paths: Option, std::option::Option, core::option::Option.
///
/// Based on https://duskmoon314.com/en/blog/2022/10/01/extract-type-from-option-in-rs-procmacro/
pub(crate) fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = ty {
        let segments_str = &path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        let option_segment = ["Option", "std::option::Option", "core::option::Option"]
            .iter()
            .find(|s| segments_str == *s)
            .and_then(|_| path.segments.last());
        let inner_type = option_segment
            .and_then(|path_seg| match &path_seg.arguments {
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) => args.first(),
                _ => None,
            })
            .and_then(|generic_arg| match generic_arg {
                syn::GenericArgument::Type(ty) => Some(ty),
                _ => None,
            });
        inner_type
    } else {
        None
    }
}

/// Extract doc comments and other documentation attributes to copy to generated code.
///
/// This preserves `#[doc = "..."]` attributes (which include `///` and `//!` comments).
pub(crate) fn extract_doc_attributes(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| {
            // Keep doc comments and other documentation attributes
            attr.path().is_ident("doc")
        })
        .cloned()
        .collect()
}

/// Extract serde attributes to copy to generated structs.
///
/// This preserves `#[serde(...)]` attributes so they can be applied to
/// generated internal structs that need serialization.
pub(crate) fn extract_serde_attributes(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| {
            // Keep serde attributes
            attr.path().is_ident("serde")
        })
        .cloned()
        .collect()
}

/// Convert snake_case to Title-Case for HTTP headers.
///
/// # Examples
///
/// - "authorization" -> "Authorization"
/// - "content_type" -> "Content-Type"
/// - "x_custom_header" -> "X-Custom-Header"
pub(crate) fn snake_to_title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

/// Convert PascalCase to snake_case.
///
/// # Examples
///
/// - "GetUser" -> "get_user"
/// - "CreateUserRequest" -> "create_user_request"
/// - "APIClient" -> "a_p_i_client"
pub(crate) fn pascal_to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_title_case() {
        assert_eq!(snake_to_title_case("authorization"), "Authorization");
        assert_eq!(snake_to_title_case("content_type"), "Content-Type");
        assert_eq!(snake_to_title_case("x_custom_header"), "X-Custom-Header");
        assert_eq!(snake_to_title_case("api_key"), "Api-Key");
    }

    #[test]
    fn test_pascal_to_snake_case() {
        assert_eq!(pascal_to_snake_case("GetUser"), "get_user");
        assert_eq!(pascal_to_snake_case("CreateUserRequest"), "create_user_request");
        assert_eq!(pascal_to_snake_case("APIClient"), "a_p_i_client");
    }
}
