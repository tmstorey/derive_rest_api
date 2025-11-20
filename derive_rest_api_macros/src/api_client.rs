//! API client generation module.
//!
//! This module generates client structs that wrap a configuration struct
//! and provide methods for making API requests using the RequestBuilder pattern.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, token::Comma, Ident, LitStr, Token};

/// Main entry point for the ApiClient derive macro
pub(crate) fn generate_api_client(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;

    // Parse the api_client attribute
    let attrs = parse_api_client_attributes(&input.attrs)?;

    // Check if this is a unit struct or empty struct (no fields)
    let is_unit_or_empty = match &input.data {
        syn::Data::Struct(data_struct) => {
            match &data_struct.fields {
                syn::Fields::Unit => true,  // Unit struct: `struct Foo;`
                syn::Fields::Named(fields) => fields.named.is_empty(),  // Empty struct: `struct Foo {}`
                syn::Fields::Unnamed(fields) => fields.unnamed.is_empty(),  // Empty tuple struct: `struct Foo();`
            }
        }
        _ => false,
    };

    // Generate automatic NoRequestConfiguration impl for unit/empty structs
    let no_config_impl = if is_unit_or_empty {
        quote! {
            impl derive_rest_api::NoRequestConfiguration for #struct_name {}
        }
    } else {
        quote! {}
    };

    // Generate client struct names
    let client_name = generate_client_name(struct_name);
    let async_client_name = generate_async_client_name(struct_name);

    // Generate the blocking client
    let blocking_client = generate_blocking_client(
        struct_name,
        &client_name,
        &attrs,
    );

    // Generate the async client
    let async_client = generate_async_client(
        struct_name,
        &async_client_name,
        &attrs,
    );

    Ok(quote! {
        #no_config_impl
        #blocking_client
        #async_client
    })
}

/// Attributes parsed from #[api_client(...)]
#[derive(Debug)]
struct ApiClientAttributes {
    base_url: String,
    requests: Vec<RequestMapping>,
}

/// Maps a request struct to a method name
#[derive(Debug)]
struct RequestMapping {
    struct_name: Ident,
    method_name: Option<String>,
}

/// Parse the #[api_client(...)] attribute
fn parse_api_client_attributes(attrs: &[syn::Attribute]) -> syn::Result<ApiClientAttributes> {
    for attr in attrs {
        if attr.path().is_ident("api_client") {
            return attr.parse_args::<ApiClientAttributes>();
        }
    }

    Err(syn::Error::new_spanned(
        attrs.first(),
        "Missing #[api_client(...)] attribute",
    ))
}

impl Parse for ApiClientAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut base_url: Option<String> = None;
        let mut requests: Option<Vec<RequestMapping>> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;

            if key == "base_url" {
                input.parse::<Token![=]>()?;
                let lit: LitStr = input.parse()?;
                base_url = Some(lit.value());
            } else if key == "requests" {
                let content;
                syn::parenthesized!(content in input);
                requests = Some(parse_request_mappings(&content)?);
            } else {
                return Err(syn::Error::new_spanned(
                    &key,
                    format!("Unknown attribute key: {}", key),
                ));
            }

            // Parse optional comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ApiClientAttributes {
            base_url: base_url.ok_or_else(|| {
                syn::Error::new(input.span(), "Missing 'base_url' attribute")
            })?,
            requests: requests.ok_or_else(|| {
                syn::Error::new(input.span(), "Missing 'requests' attribute")
            })?,
        })
    }
}

/// Parse request mappings like: GetUser, CreateUser = "new_user"
fn parse_request_mappings(input: syn::parse::ParseStream) -> syn::Result<Vec<RequestMapping>> {
    let punct = Punctuated::<RequestMapping, Comma>::parse_terminated(input)?;
    Ok(punct.into_iter().collect())
}

impl Parse for RequestMapping {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let struct_name: Ident = input.parse()?;

        let method_name = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let lit: LitStr = input.parse()?;
            Some(lit.value())
        } else {
            None
        };

        Ok(RequestMapping {
            struct_name,
            method_name,
        })
    }
}

/// Generate client struct name by stripping "Config" suffix if present
fn generate_client_name(struct_name: &Ident) -> Ident {
    let name = struct_name.to_string();
    let base_name = name.strip_suffix("Config").unwrap_or(&name);
    quote::format_ident!("{}Client", base_name)
}

/// Generate async client struct name
fn generate_async_client_name(struct_name: &Ident) -> Ident {
    let name = struct_name.to_string();
    let base_name = name.strip_suffix("Config").unwrap_or(&name);
    quote::format_ident!("{}AsyncClient", base_name)
}

/// Convert struct name to snake_case method name
fn struct_name_to_method_name(struct_name: &Ident) -> String {
    let name = struct_name.to_string();

    // Convert PascalCase to snake_case
    let mut result = String::new();
    let mut chars = name.chars().peekable();

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

/// Generate the blocking client struct and impl
fn generate_blocking_client(
    config_struct: &Ident,
    client_name: &Ident,
    attrs: &ApiClientAttributes,
) -> TokenStream {
    let base_url = &attrs.base_url;

    // Generate methods for each request
    let methods: Vec<_> = attrs.requests.iter().map(|mapping| {
        let struct_name = &mapping.struct_name;
        let builder_name = quote::format_ident!("{}Builder", struct_name);

        let method_name = mapping.method_name.as_ref()
            .map(|s| quote::format_ident!("{}", s))
            .unwrap_or_else(|| {
                let name = struct_name_to_method_name(struct_name);
                quote::format_ident!("{}", name)
            });

        quote! {
            #[doc = concat!("Creates a new [`", stringify!(#struct_name), "`] request builder.")]
            #[doc = ""]
            #[doc = "The builder is pre-configured with the client's HTTP client and base URL."]
            #[doc = "If the config implements `ConfigureRequest`, it will also be pre-configured with those settings."]
            pub fn #method_name(&self) -> #builder_name<C, ()> {
                let builder = #builder_name::new()
                    .http_client((&self.client).clone())
                    .base_url(&self.base_url);

                // Apply configuration if the config implements ConfigureRequest
                if let std::option::Option::Some(config) = &self.config {
                    <#config_struct as derive_rest_api::ConfigureRequest>::configure(config, builder)
                } else {
                    builder
                }
            }
        }
    }).collect();

    quote! {
        #[doc = concat!("Blocking HTTP client for [`", stringify!(#config_struct), "`].")]
        pub struct #client_name<C: derive_rest_api::HttpClient> {
            config: std::option::Option<#config_struct>,
            base_url: std::string::String,
            client: C,
        }

        impl<C: derive_rest_api::HttpClient> #client_name<C> {
            #[doc = concat!("Creates a new [`", stringify!(#client_name), "`].")]
            pub fn new() -> Self {
                let client = C::default();
                Self {
                    config: std::option::Option::None,
                    base_url: #base_url.to_string(),
                    client,
                }
            }

            #[doc = "Sets a custom base URL for this client."]
            pub fn with_base_url(mut self, base_url: impl std::convert::Into<std::string::String>) -> Self {
                self.base_url = base_url.into();
                self
            }

            #[doc = "Sets the underlying HTTP client for this API client."]
            pub fn with_http_client(mut self, client: impl std::convert::Into<C>) -> Self {
                self.client = client.into();
                self
            }

            #[doc = "Sets the config for this client."]
            pub fn with_config(mut self, config: #config_struct) -> Self {
                self.config = std::option::Option::Some(config);
                self
            }

            #[doc = "Returns a reference to the configuration."]
            pub fn config(&self) -> &std::option::Option<#config_struct> {
                &self.config
            }

            #(#methods)*
        }
    }
}

/// Generate the async client struct and impl
fn generate_async_client(
    config_struct: &Ident,
    client_name: &Ident,
    attrs: &ApiClientAttributes,
) -> TokenStream {
    let base_url = &attrs.base_url;

    // Generate methods for each request
    let methods: Vec<_> = attrs.requests.iter().map(|mapping| {
        let struct_name = &mapping.struct_name;
        let builder_name = quote::format_ident!("{}Builder", struct_name);

        let method_name = mapping.method_name.as_ref()
            .map(|s| quote::format_ident!("{}", s))
            .unwrap_or_else(|| {
                let name = struct_name_to_method_name(struct_name);
                quote::format_ident!("{}", name)
            });

        quote! {
            #[doc = concat!("Creates a new [`", stringify!(#struct_name), "`] request builder.")]
            #[doc = ""]
            #[doc = "The builder is pre-configured with the client's async HTTP client and base URL."]
            #[doc = "If the config implements `ConfigureRequest`, it will also be pre-configured with those settings."]
            pub fn #method_name(&self) -> #builder_name<(), A> {
                let builder = #builder_name::new()
                    .async_http_client((&self.client).clone())
                    .base_url(&self.base_url);

                // Apply configuration if the config implements ConfigureRequest
                if let std::option::Option::Some(config) = &self.config {
                    <#config_struct as derive_rest_api::ConfigureRequest>::configure(config, builder)
                } else {
                    builder
                }
            }
        }
    }).collect();

    quote! {
        #[doc = concat!("Async HTTP client for [`", stringify!(#config_struct), "`].")]
        pub struct #client_name<A: derive_rest_api::AsyncHttpClient> {
            config: std::option::Option<#config_struct>,
            base_url: std::string::String,
            client: A,
        }

        impl<A: derive_rest_api::AsyncHttpClient> #client_name<A> {
            #[doc = concat!("Creates a new [`", stringify!(#client_name), "`].")]
            pub fn new() -> Self {
                let client = A::default();
                Self {
                    config: std::option::Option::None,
                    base_url: #base_url.to_string(),
                    client,
                }
            }

            #[doc = "Sets a custom base URL for this client."]
            pub fn with_base_url(mut self, base_url: impl std::convert::Into<std::string::String>) -> Self {
                self.base_url = base_url.into();
                self
            }

            #[doc = "Sets the underlying HTTP client for this API client."]
            pub fn with_http_client(mut self, client: impl std::convert::Into<A>) -> Self {
                self.client = client.into();
                self
            }

            #[doc = "Sets the config for this client."]
            pub fn with_config(mut self, config: #config_struct) -> Self {
                self.config = std::option::Option::Some(config);
                self
            }

            #[doc = "Returns a reference to the configuration."]
            pub fn config(&self) -> &std::option::Option<#config_struct> {
                &self.config
            }

            #(#methods)*
        }
    }
}
