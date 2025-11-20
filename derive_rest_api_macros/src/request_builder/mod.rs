//! RequestBuilder derive macro implementation.
//!
//! This module generates a builder pattern for REST API requests, including:
//! - A builder struct with optional fields
//! - Setter methods for each field
//! - A `build()` method that validates and constructs the original struct
//! - HTTP methods (`build_url`, `build_body`, `build_headers`, `send_with_client`)
//! - Convenience methods (`send`, `send_async`) when clients are embedded

mod attributes;
mod builder;
mod http;
mod utils;

use crate::utils::extract_doc_attributes;
use attributes::parse_struct_attributes;
use builder::{
    generate_build_fields, generate_builder_fields, generate_builder_send_methods,
    generate_field_processing, generate_setter_methods,
};
use http::generate_http_methods_impl;
use quote::quote;
use syn;

/// Main entry point for generating the RequestBuilder derive macro code.
///
/// This function orchestrates the generation of all builder-related code by
/// delegating to specialized generation functions.
pub(crate) fn generate_request_builder(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    // Extract the struct name
    let struct_name = &input.ident;
    let builder_name = quote::format_ident!("{}Builder", struct_name);

    // Parse struct-level attributes
    let struct_attrs = parse_struct_attributes(&input.attrs)?;

    // Extract doc comments and other attributes to copy to the builder
    let struct_doc_attrs = extract_doc_attributes(&input.attrs);

    // Extract fields from the struct
    let fields = match &input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "RequestBuilder only supports structs with named fields",
                ))
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "RequestBuilder can only be derived for structs",
            ))
        }
    };

    // Generate builder struct fields
    let builder_fields = generate_builder_fields(fields);

    // Generate field names for constructor (collect to allow multiple uses)
    let field_names: Vec<_> = fields.iter().map(|field| &field.ident).collect();

    // Generate setter methods for each field
    let setter_methods = generate_setter_methods(fields, &struct_attrs);

    // Generate field extraction and validation for build() method
    let field_processing = generate_field_processing(fields, &struct_attrs);

    // Generate final field assignments using temporary variables
    let build_fields = generate_build_fields(fields);

    // Generate HTTP methods impl block (build_url, build_body, build_headers, send_with_client)
    let http_methods_impl = generate_http_methods_impl(struct_name, fields, &struct_attrs);

    // Generate send() and send_async() methods if path is present
    let send_methods = if struct_attrs.path.is_some() {
        generate_builder_send_methods(struct_name, &struct_attrs)
    } else {
        quote! {}
    };

    // Generate the builder struct and its impl block
    let expanded = quote! {
        #(#struct_doc_attrs)*
        #[doc = ""]
        #[doc = concat!("Builder for [`", stringify!(#struct_name), "`].")]
        pub struct #builder_name<__C = (), __A = ()> {
            #(#builder_fields),*,
            __http_client: std::option::Option<__C>,
            __async_http_client: std::option::Option<__A>,
            __base_url: std::option::Option<std::string::String>,
            __dynamic_headers: std::collections::HashMap<std::string::String, std::string::String>,
            __timeout: std::option::Option<std::time::Duration>,
        }

        impl #builder_name<(), ()> {
            #[doc = concat!("Creates a new [`", stringify!(#builder_name), "`] with all fields set to `None`.")]
            pub fn new() -> Self {
                Self {
                    #(#field_names: std::option::Option::None),*,
                    __http_client: std::option::Option::None,
                    __async_http_client: std::option::Option::None,
                    __base_url: std::option::Option::None,
                    __dynamic_headers: std::collections::HashMap::new(),
                    __timeout: std::option::Option::None,
                }
            }
        }

        impl<__C, __A> #builder_name<__C, __A> {
            #[doc = "Sets the HTTP client to use for blocking requests."]
            pub fn http_client<C2: derive_rest_api::HttpClient>(self, client: C2) -> #builder_name<C2, __A> {
                #builder_name {
                    #(#field_names: self.#field_names),*,
                    __http_client: std::option::Option::Some(client),
                    __async_http_client: self.__async_http_client,
                    __base_url: self.__base_url,
                    __dynamic_headers: self.__dynamic_headers,
                    __timeout: self.__timeout,
                }
            }

            #[doc = "Sets the async HTTP client to use for async requests."]
            pub fn async_http_client<A2: derive_rest_api::AsyncHttpClient>(self, client: A2) -> #builder_name<__C, A2> {
                #builder_name {
                    #(#field_names: self.#field_names),*,
                    __http_client: self.__http_client,
                    __async_http_client: std::option::Option::Some(client),
                    __base_url: self.__base_url,
                    __dynamic_headers: self.__dynamic_headers,
                    __timeout: self.__timeout,
                }
            }

            #[doc = "Sets the base URL for the request."]
            pub fn base_url(mut self, base_url: impl std::convert::Into<std::string::String>) -> Self {
                self.__base_url = std::option::Option::Some(base_url.into());
                self
            }
        }

        // Implement RequestModifier trait for the builder
        impl<__C, __A> derive_rest_api::RequestModifier for #builder_name<__C, __A> {
            fn header(mut self, name: impl std::convert::Into<std::string::String>, value: impl std::convert::Into<std::string::String>) -> Self {
                self.__dynamic_headers.insert(name.into(), value.into());
                self
            }

            fn timeout(mut self, timeout: std::time::Duration) -> Self {
                self.__timeout = std::option::Option::Some(timeout);
                self
            }
        }

        impl<__C, __A> #builder_name<__C, __A> {

            #(#setter_methods)*

            #[doc = concat!("Builds a [`", stringify!(#struct_name), "`] from the builder.")]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = ""]
            #[doc = "Returns an error if any required fields are not set or if validation fails."]
            pub fn build(self) -> std::result::Result<#struct_name, derive_rest_api::RequestError> {
                // Extract and validate fields
                #(#field_processing)*

                // Construct the struct
                std::result::Result::Ok(#struct_name {
                    #(#build_fields),*
                })
            }
        }

        // Generate send() and send_async() methods for builder
        #send_methods

        // Generate HTTP methods impl for the original struct
        #http_methods_impl
    };

    Ok(expanded)
}
