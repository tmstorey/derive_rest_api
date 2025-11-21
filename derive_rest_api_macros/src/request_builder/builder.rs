//! Builder struct and methods generation.
//!
//! This module generates the builder struct, setter methods, field processing,
//! and the send/send_async methods that use embedded HTTP clients.

use crate::utils::{extract_doc_attributes, option_inner_type};
use super::attributes::{StructAttributes, parse_field_attributes, DefaultBehavior};
use proc_macro2::TokenStream;
use quote::quote;
use syn;

/// Generate builder struct field definitions
/// Wraps non-Option fields in Option, keeps Option fields as-is
pub(super) fn generate_builder_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        // Check if this field is already an Option
        if option_inner_type(field_type).is_some() {
            // Already Option, don't wrap again
            quote! {
                #field_name: #field_type
            }
        } else {
            // Not an Option, wrap it
            quote! {
                #field_name: std::option::Option<#field_type>
            }
        }
    })
}

/// Generate setter methods for builder fields
pub(super) fn generate_setter_methods<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_attrs: &'a StructAttributes,
) -> impl Iterator<Item = TokenStream> + 'a {
    fields.iter().map(move |field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        // Parse field-level attributes
        let field_attrs = parse_field_attributes(&field.attrs).unwrap_or_default();
        let use_into = struct_attrs.into || field_attrs.into;

        // Extract doc comments from the field
        let field_doc_attrs = extract_doc_attributes(&field.attrs);

        // Check if this field is already an Option
        if let Some(inner_type) = option_inner_type(field_type) {
            // Field is Option<T>, setter takes T (or impl Into<T>) and wraps it
            if use_into {
                quote! {
                    #(#field_doc_attrs)*
                    pub fn #field_name(mut self, value: impl std::convert::Into<#inner_type>) -> Self {
                        self.#field_name = std::option::Option::Some(value.into());
                        self
                    }
                }
            } else {
                quote! {
                    #(#field_doc_attrs)*
                    pub fn #field_name(mut self, value: #inner_type) -> Self {
                        self.#field_name = std::option::Option::Some(value);
                        self
                    }
                }
            }
        } else {
            // Field is not Option, setter takes the type directly (or impl Into<T>)
            if use_into {
                quote! {
                    #(#field_doc_attrs)*
                    pub fn #field_name(mut self, value: impl std::convert::Into<#field_type>) -> Self {
                        self.#field_name = std::option::Option::Some(value.into());
                        self
                    }
                }
            } else {
                quote! {
                    #(#field_doc_attrs)*
                    pub fn #field_name(mut self, value: #field_type) -> Self {
                        self.#field_name = std::option::Option::Some(value);
                        self
                    }
                }
            }
        }
    })
}

/// Generate field processing code for the build() method
/// Handles extraction, validation, and default values
pub(super) fn generate_field_processing<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_attrs: &'a StructAttributes,
) -> impl Iterator<Item = TokenStream> + 'a {
    fields.iter().map(move |field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let field_name_str = field_name.as_ref().unwrap().to_string();
        let temp_var = quote::format_ident!("__field_{}", field_name.as_ref().unwrap());

        // Parse field-level attributes
        let field_attrs = parse_field_attributes(&field.attrs).unwrap_or_default();

        // Determine the default behavior for this field
        let default_behavior = match &field_attrs.default {
            DefaultBehavior::Required => {
                // If struct has default attribute, use Default::default()
                if struct_attrs.default {
                    DefaultBehavior::UseDefault
                } else {
                    DefaultBehavior::Required
                }
            },
            // Field-level default overrides struct-level
            other => other.clone(),
        };

        let validate_fn = field_attrs.validate.as_ref();

        // Generate value extraction
        let value_extraction = if option_inner_type(field_type).is_some() {
            // Field is already Option, just use it
            quote! {
                let #temp_var = self.#field_name;
            }
        } else {
            match &default_behavior {
                DefaultBehavior::Required => {
                    // Field is required, error if not set
                    quote! {
                        let #temp_var = self.#field_name.ok_or_else(|| derive_rest_api::RequestError::missing_field(#field_name_str))?;
                    }
                },
                DefaultBehavior::UseDefault => {
                    // Use Default::default() if not set
                    quote! {
                        let #temp_var = self.#field_name.unwrap_or_default();
                    }
                },
                DefaultBehavior::Custom(expr) => {
                    // Use custom expression if not set
                    quote! {
                        let #temp_var = self.#field_name.unwrap_or_else(|| #expr);
                    }
                },
            }
        };

        // Generate validation if needed
        let validation = if let Some(validate_fn) = validate_fn {
            if option_inner_type(field_type).is_some() {
                // Optional field: validate if Some
                quote! {
                    if let std::option::Option::Some(ref value) = #temp_var {
                        #validate_fn(value).map_err(|e| derive_rest_api::RequestError::validation_error(#field_name_str, e))?;
                    }
                }
            } else {
                // Non-optional field: always validate
                quote! {
                    #validate_fn(&#temp_var).map_err(|e| derive_rest_api::RequestError::validation_error(#field_name_str, e))?;
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #value_extraction
            #validation
        }
    })
}

/// Generate final field assignments for struct construction
pub(super) fn generate_build_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> impl Iterator<Item = TokenStream> + '_ {
    fields.iter().map(|field| {
        let field_name = &field.ident;
        let temp_var = quote::format_ident!("__field_{}", field_name.as_ref().unwrap());

        quote! {
            #field_name: #temp_var
        }
    })
}

/// Generate send() and send_async() methods for the builder
pub(super) fn generate_builder_send_methods(
    struct_name: &syn::Ident,
    struct_attrs: &StructAttributes,
) -> TokenStream {
    let builder_name = quote::format_ident!("{}Builder", struct_name);
    let method_value = struct_attrs.method.as_ref().map(|s| s.as_str()).unwrap_or("GET");
    let return_type = struct_attrs.response.clone().unwrap_or(syn::Type::Verbatim(quote! {Vec<u8>}));

    let return_value = match struct_attrs.response.clone() {
        Some(_) => quote! {
            let bytes = response?;
            serde_json::from_slice(&bytes)
                .map_err(|e| derive_rest_api::RequestError::ResponseDeserializationError { source: e })
        },
        _ => quote! { response },
    };

    quote! {
        // Impl block for builders with an HTTP client
        impl<__C: derive_rest_api::HttpClient, __A> #builder_name<__C, __A> {
            #[doc = "Builds the request and sends it using the embedded HTTP client."]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = ""]
            #[doc = "Returns an error if:"]
            #[doc = "- No base URL is configured (use `.base_url()` to set one)"]
            #[doc = "- Building the request fails (missing required fields, validation errors)"]
            #[doc = "- URL building fails"]
            #[doc = "- Body serialization fails"]
            #[doc = "- The HTTP request fails"]
            pub fn send(mut self) -> std::result::Result<#return_type, derive_rest_api::RequestError> {
                // Extract client and base URL before building
                let client = self.__http_client.take()
                    .ok_or_else(|| derive_rest_api::RequestError::missing_field("http_client"))?;

                let base_url = self.__base_url.take()
                    .ok_or_else(|| derive_rest_api::RequestError::MissingBaseUrl)?;

                let timeout = self.__timeout.take();
                let dynamic_headers = self.__dynamic_headers.clone();
                let request = self.build()?;
                let path = request.build_url().map_err(|e| derive_rest_api::RequestError::UrlBuildError { source: std::boxed::Box::new(e) })?;
                let url = format!("{}{}", base_url, path);
                let mut headers = request.build_headers();
                // Merge dynamic headers (these override request headers)
                headers.extend(dynamic_headers);
                let body = request.build_body()?;

                let response = client.send(#method_value, &url, headers, body, timeout)
                    .map_err(|e| derive_rest_api::RequestError::http_error(e));

                #return_value
            }
        }

        // Impl block for builders with an async HTTP client
        impl<__C, __A: derive_rest_api::AsyncHttpClient> #builder_name<__C, __A> {
            #[doc = "Builds the request and sends it using the embedded async HTTP client."]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = ""]
            #[doc = "Returns an error if:"]
            #[doc = "- No base URL is configured (use `.base_url()` to set one)"]
            #[doc = "- Building the request fails (missing required fields, validation errors)"]
            #[doc = "- URL building fails"]
            #[doc = "- Body serialization fails"]
            #[doc = "- The HTTP request fails"]
            pub async fn send_async(mut self) -> std::result::Result<#return_type, derive_rest_api::RequestError> {
                // Extract client and base URL before building
                let client = self.__async_http_client.take()
                    .ok_or_else(|| derive_rest_api::RequestError::missing_field("async_http_client"))?;

                let base_url = self.__base_url.take()
                    .ok_or_else(|| derive_rest_api::RequestError::MissingBaseUrl)?;

                let timeout = self.__timeout.take();
                let dynamic_headers = self.__dynamic_headers.clone();
                let request = self.build()?;
                let path = request.build_url().map_err(|e| derive_rest_api::RequestError::UrlBuildError { source: std::boxed::Box::new(e) })?;
                let url = format!("{}{}", base_url, path);
                let mut headers = request.build_headers();
                // Merge dynamic headers (these override request headers)
                headers.extend(dynamic_headers);
                let body = request.build_body()?;

                let response = client.send_async(#method_value, &url, headers, body, timeout).await
                    .map_err(|e| derive_rest_api::RequestError::http_error(e));

                #return_value
            }
        }
    }
}
