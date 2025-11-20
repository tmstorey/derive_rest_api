//! Attribute parsing for the RequestBuilder derive macro.
//!
//! This module defines the attribute structures and parsing logic for both
//! struct-level and field-level `#[request_builder(...)]` attributes.

use syn;

/// Struct-level attributes from #[request_builder(...)]
#[derive(Debug, Default)]
pub(super) struct StructAttributes {
    /// Use Into<T> for all setters
    pub into: bool,
    /// Use Default::default() for all fields if not set
    pub default: bool,
    /// HTTP method (GET, POST, PUT, DELETE, PATCH, etc.)
    pub method: Option<String>,
    /// URL path (e.g., "/api/users/{id}")
    pub path: Option<String>,
    /// Query string config expression (e.g., "my_qs_config()")
    pub query_config: Option<String>,
    /// Response type
    pub response: Option<syn::Type>,
}

/// Field-level attributes from #[request_builder(...)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldKind {
    #[default]
    Unspecified,
    /// Field goes in the URL path (e.g., {id})
    Path,
    /// Field goes in query string
    Query,
    /// Field goes in request body
    Body,
    /// Field goes in HTTP header
    Header,
}

/// Field-level attributes
#[derive(Debug, Default)]
pub(super) struct FieldAttributes {
    /// Use Into<T> for this setter
    pub into: bool,
    /// Use Default::default() if not set
    pub default: bool,
    /// Validation function path (e.g., validate_email)
    pub validate: Option<syn::Path>,
    /// Where this field should go in the request
    pub kind: FieldKind,
    /// Custom name for this field (for headers, query params, etc.)
    pub rename: Option<String>,
}

/// Parse struct-level #[request_builder(...)] attributes
pub(super) fn parse_struct_attributes(attrs: &[syn::Attribute]) -> syn::Result<StructAttributes> {
    let mut result = StructAttributes::default();

    for attr in attrs {
        if !attr.path().is_ident("request_builder") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            // #[request_builder(into)]
            if meta.path.is_ident("into") {
                result.into = true;
                return Ok(());
            }

            // #[request_builder(method = "GET")]
            if meta.path.is_ident("method") {
                let value = meta.value()?;
                let method: syn::LitStr = value.parse()?;
                result.method = Some(method.value());
                return Ok(());
            }

            // #[request_builder(path = "/api/users/{id}")]
            if meta.path.is_ident("path") {
                let value = meta.value()?;
                let path: syn::LitStr = value.parse()?;
                result.path = Some(path.value());
                return Ok(());
            }

            // #[request_builder(response = User)]
            if meta.path.is_ident("response") {
                let value = meta.value()?;
                let response_type: syn::Type = value.parse()?;
                result.response = Some(response_type);
                return Ok(());
            }

            // #[request_builder(default)]
            if meta.path.is_ident("default") {
                result.default = true;
                return Ok(());
            }

            // #[request_builder(query_config = "my_qs_config()")]
            if meta.path.is_ident("query_config") {
                let value = meta.value()?;
                let config_expr: syn::LitStr = value.parse()?;
                result.query_config = Some(config_expr.value());
                return Ok(());
            }

            Err(meta.error("unsupported request_builder attribute"))
        })?;
    }

    Ok(result)
}

/// Parse field-level #[request_builder(...)] attributes
pub(super) fn parse_field_attributes(attrs: &[syn::Attribute]) -> syn::Result<FieldAttributes> {
    let mut result = FieldAttributes::default();

    for attr in attrs {
        if !attr.path().is_ident("request_builder") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            // #[request_builder(into)]
            if meta.path.is_ident("into") {
                result.into = true;
                return Ok(());
            }

            // #[request_builder(path)]
            if meta.path.is_ident("path") {
                result.kind = FieldKind::Path;
                return Ok(());
            }

            // #[request_builder(query)] or #[request_builder(query = "name")]
            if meta.path.is_ident("query") {
                result.kind = FieldKind::Query;
                if meta.input.peek(syn::Token![=]) {
                    let value = meta.value()?;
                    let name: syn::LitStr = value.parse()?;
                    result.rename = Some(name.value());
                }
                return Ok(());
            }

            // #[request_builder(body)] or #[request_builder(body = "name")]
            if meta.path.is_ident("body") {
                result.kind = FieldKind::Body;
                if meta.input.peek(syn::Token![=]) {
                    let value = meta.value()?;
                    let name: syn::LitStr = value.parse()?;
                    result.rename = Some(name.value());
                }
                return Ok(());
            }

            // #[request_builder(header)] or #[request_builder(header = "Authorization")]
            if meta.path.is_ident("header") {
                result.kind = FieldKind::Header;
                if meta.input.peek(syn::Token![=]) {
                    let value = meta.value()?;
                    let name: syn::LitStr = value.parse()?;
                    result.rename = Some(name.value());
                }
                return Ok(());
            }

            // #[request_builder(default)]
            if meta.path.is_ident("default") {
                result.default = true;
                return Ok(());
            }

            // #[request_builder(validate = "function_path")]
            if meta.path.is_ident("validate") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                let path: syn::Path = lit.parse()?;
                result.validate = Some(path);
                return Ok(());
            }

            Err(meta.error("unsupported field-level request_builder attribute"))
        })?;
    }

    Ok(result)
}
