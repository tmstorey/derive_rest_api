//! HTTP request methods generation.
//!
//! This module generates the HTTP-related methods on the request struct,
//! including build_url, build_body, build_headers, and send_with_client.

use crate::utils::{extract_serde_attributes, option_inner_type, snake_to_title_case};
use super::attributes::{FieldKind, StructAttributes, parse_field_attributes};
use super::utils::extract_path_params;
use proc_macro2::TokenStream;
use quote::quote;
use syn;

/// Generate the impl block with HTTP-related methods (build_url, build_body, build_headers, send_with_client)
pub(super) fn generate_http_methods_impl(
    struct_name: &syn::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_attrs: &StructAttributes,
) -> TokenStream {
    if let Some(path_template) = &struct_attrs.path {
        let path_params: Vec<String> = extract_path_params(path_template);

        let query_fields: Vec<_> = fields.iter().filter(|field| {
            parse_field_attributes(&field.attrs)
                .map(|attrs| attrs.kind == FieldKind::Query)
                .unwrap_or(false)
        }).collect();

        let body_fields: Vec<_> = fields.iter().filter(|field| {
            parse_field_attributes(&field.attrs)
                .map(|attrs| attrs.kind == FieldKind::Body)
                .unwrap_or(false)
        }).collect();

        let header_fields: Vec<_> = fields.iter().filter(|field| {
            parse_field_attributes(&field.attrs)
                .map(|attrs| attrs.kind == FieldKind::Header)
                .unwrap_or(false)
        }).collect();

        let path_replacements = generate_path_replacements(&path_params, fields);
        let query_serialization = generate_query_serialization(&query_fields, struct_attrs);
        let build_body_method = generate_build_body_method(&body_fields);
        let build_headers_method = generate_request_build_headers_method(&header_fields);
        let send_with_client_method = generate_send_with_client_method(struct_attrs);

        quote! {
            impl #struct_name {
                #[doc = "Builds the URL path by substituting path parameters and appending query string."]
                #[doc = ""]
                #[doc = "# Errors"]
                #[doc = ""]
                #[doc = "Returns an error if any required path parameters are not set or if query serialization fails."]
                pub fn build_url(&self) -> std::result::Result<std::string::String, derive_rest_api::RequestError> {
                    let mut path = std::string::String::from(#path_template);
                    #(#path_replacements)*
                    #query_serialization
                    std::result::Result::Ok(path)
                }

                #build_body_method

                #build_headers_method

                #send_with_client_method
            }
        }
    } else {
        quote! {}
    }
}

/// Generate path parameter replacement code
fn generate_path_replacements(
    path_params: &[String],
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<TokenStream> {
    path_params.iter().map(|param| {
        let matching_field = fields.iter().find(|field| {
            field.ident.as_ref().unwrap().to_string() == *param
        });

        if let Some(field) = matching_field {
            let field_name = &field.ident;
            let placeholder = format!("{{{}}}", param);
            let is_option = option_inner_type(&field.ty).is_some();

            if is_option {
                quote! {
                    path = path.replace(#placeholder, &self.#field_name
                        .as_ref()
                        .ok_or_else(|| derive_rest_api::RequestError::missing_path_parameter(#param))?
                        .to_string());
                }
            } else {
                quote! {
                    path = path.replace(#placeholder, &self.#field_name.to_string());
                }
            }
        } else {
            quote! {
                compile_error!(concat!("Path parameter '", #param, "' does not match any field"));
            }
        }
    }).collect()
}

/// Generate query string serialization code
fn generate_query_serialization(
    query_fields: &[&syn::Field],
    struct_attrs: &StructAttributes,
) -> TokenStream {
    if query_fields.is_empty() {
        return quote! {};
    }

    let query_struct_fields = query_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let serde_attrs = extract_serde_attributes(&field.attrs);

        let skip_attr = if option_inner_type(field_type).is_some() {
            quote! { #[serde(skip_serializing_if = "Option::is_none")] }
        } else {
            quote! {}
        };

        quote! {
            #(#serde_attrs)*
            #skip_attr
            #field_name: #field_type
        }
    });

    let query_field_assignments = query_fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { #field_name: self.#field_name.clone() }
    });

    let config_expr = if let Some(config) = &struct_attrs.query_config {
        let config_tokens: TokenStream = config.parse().unwrap();
        quote! { #config_tokens }
    } else {
        quote! { serde_qs::Config::new() }
    };

    quote! {
        #[derive(serde::Serialize)]
        struct QueryParams {
            #(#query_struct_fields),*
        }

        let query_params = QueryParams {
            #(#query_field_assignments),*
        };

        let config = #config_expr;
        let query_string = config.serialize_string(&query_params)
            .map_err(|e| derive_rest_api::RequestError::QuerySerializationError { source: e })?;

        if !query_string.is_empty() {
            path.push('?');
            path.push_str(&query_string);
        }
    }
}

/// Generate the build_body() method
fn generate_build_body_method(body_fields: &[&syn::Field]) -> TokenStream {
    if body_fields.is_empty() {
        return quote! {
            #[doc = "Builds the request body (always returns None as there are no body fields)."]
            pub fn build_body(&self) -> std::result::Result<std::option::Option<std::vec::Vec<u8>>, derive_rest_api::RequestError> {
                std::result::Result::Ok(std::option::Option::None)
            }
        };
    }

    let body_struct_fields = body_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let serde_attrs = extract_serde_attributes(&field.attrs);

        let skip_attr = if option_inner_type(field_type).is_some() {
            quote! { #[serde(skip_serializing_if = "Option::is_none")] }
        } else {
            quote! {}
        };

        quote! {
            #(#serde_attrs)*
            #skip_attr
            #field_name: #field_type
        }
    });

    let body_field_assignments = body_fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { #field_name: self.#field_name.clone() }
    });

    quote! {
        #[doc = "Builds the request body as JSON."]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an error if JSON serialization fails."]
        pub fn build_body(&self) -> std::result::Result<std::option::Option<std::vec::Vec<u8>>, derive_rest_api::RequestError> {
            #[derive(serde::Serialize)]
            struct BodyParams {
                #(#body_struct_fields),*
            }

            let body_params = BodyParams {
                #(#body_field_assignments),*
            };

            let json = serde_json::to_vec(&body_params)
                .map_err(|e| derive_rest_api::RequestError::BodySerializationError { source: e })?;

            std::result::Result::Ok(std::option::Option::Some(json))
        }
    }
}

/// Generate the build_headers() method for the request struct (no dynamic headers)
fn generate_request_build_headers_method(header_fields: &[&syn::Field]) -> TokenStream {
    let header_insertions = header_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let field_name_str = field_name.as_ref().unwrap().to_string();

        let field_attrs = parse_field_attributes(&field.attrs).unwrap_or_default();
        let header_name = field_attrs.rename
            .unwrap_or_else(|| snake_to_title_case(&field_name_str));

        if option_inner_type(field_type).is_some() {
            quote! {
                if let std::option::Option::Some(ref value) = self.#field_name {
                    headers.insert(#header_name.to_string(), value.to_string());
                }
            }
        } else {
            quote! {
                headers.insert(#header_name.to_string(), self.#field_name.to_string());
            }
        }
    });

    quote! {
        #[doc = "Builds HTTP headers from header-annotated fields."]
        pub fn build_headers(&self) -> std::collections::HashMap<std::string::String, std::string::String> {
            let mut headers = std::collections::HashMap::new();
            #(#header_insertions)*
            headers
        }
    }
}


/// Generate the send_with_client() method
fn generate_send_with_client_method(struct_attrs: &StructAttributes) -> TokenStream {
    let method_value = struct_attrs.method.as_ref().map(|s| s.as_str()).unwrap_or("GET");

    quote! {
        #[doc = "Sends the HTTP request using the provided client."]
        #[doc = ""]
        #[doc = "# Arguments"]
        #[doc = ""]
        #[doc = "- `client`: An implementation of the `HttpClient` trait"]
        #[doc = "- `base_url`: The base URL to prepend to the request path"]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an error if URL building, body serialization, or the HTTP request fails."]
        pub fn send_with_client<C: derive_rest_api::HttpClient>(
            &self,
            client: &C,
            base_url: &str,
        ) -> std::result::Result<std::vec::Vec<u8>, derive_rest_api::RequestError> {
            let path = self.build_url().map_err(|e| derive_rest_api::RequestError::UrlBuildError { source: std::boxed::Box::new(e) })?;
            let url = format!("{}{}", base_url, path);
            let headers = self.build_headers();
            let body = self.build_body()?;

            client.send(#method_value, &url, headers, body)
                .map_err(|e| derive_rest_api::RequestError::http_error(e))
        }
    }
}
