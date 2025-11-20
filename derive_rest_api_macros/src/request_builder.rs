
use syn;
use quote;

/// Struct-level attributes from #[request_builder(...)]
#[derive(Debug, Default)]
struct StructAttributes {
    /// Use Into<T> for all setters
    into: bool,
    /// Use Default::default() for all fields if not set
    default: bool,
    /// HTTP method (GET, POST, PUT, DELETE, PATCH, etc.)
    method: Option<String>,
    /// URL path (e.g., "/api/users/{id}")
    path: Option<String>,
    /// Query string config expression (e.g., "my_qs_config()")
    query_config: Option<String>,
    /// Response type
    response: Option<syn::Type>,
}

/// Field-level attributes from #[request_builder(...)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum FieldKind {
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
struct FieldAttributes {
    /// Use Into<T> for this setter
    into: bool,
    /// Use Default::default() if not set
    default: bool,
    /// Validation function path (e.g., validate_email)
    validate: Option<syn::Path>,
    /// Where this field should go in the request
    kind: FieldKind,
    /// Custom name for this field (for headers, query params, etc.)
    rename: Option<String>,
}

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
        quote::quote! {}
    };

    // Generate the builder struct and its impl block
    let expanded = quote::quote! {
        #(#struct_doc_attrs)*
        #[doc = ""]
        #[doc = concat!("Builder for [`", stringify!(#struct_name), "`].")]
        pub struct #builder_name<__C = (), __A = ()> {
            #(#builder_fields),*,
            __http_client: std::option::Option<__C>,
            __async_http_client: std::option::Option<__A>,
            __base_url: std::option::Option<std::string::String>,
        }

        impl #builder_name<(), ()> {
            #[doc = concat!("Creates a new [`", stringify!(#builder_name), "`] with all fields set to `None`.")]
            pub fn new() -> Self {
                Self {
                    #(#field_names: std::option::Option::None),*,
                    __http_client: std::option::Option::None,
                    __async_http_client: std::option::Option::None,
                    __base_url: std::option::Option::None,
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
                }
            }

            #[doc = "Sets the async HTTP client to use for async requests."]
            pub fn async_http_client<A2: derive_rest_api::AsyncHttpClient>(self, client: A2) -> #builder_name<__C, A2> {
                #builder_name {
                    #(#field_names: self.#field_names),*,
                    __http_client: self.__http_client,
                    __async_http_client: std::option::Option::Some(client),
                    __base_url: self.__base_url,
                }
            }

            #[doc = "Sets the base URL for the request."]
            pub fn base_url(mut self, base_url: impl std::convert::Into<std::string::String>) -> Self {
                self.__base_url = std::option::Option::Some(base_url.into());
                self
            }

            #(#setter_methods)*

            #[doc = concat!("Builds a [`", stringify!(#struct_name), "`] from the builder.")]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = ""]
            #[doc = "Returns an error if any required fields are not set or if validation fails."]
            pub fn build(self) -> std::result::Result<#struct_name, std::string::String> {
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

/// Generate builder struct field definitions
/// Wraps non-Option fields in Option, keeps Option fields as-is
fn generate_builder_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        // Check if this field is already an Option
        if option_inner_type(field_type).is_some() {
            // Already Option, don't wrap again
            quote::quote! {
                #field_name: #field_type
            }
        } else {
            // Not an Option, wrap it
            quote::quote! {
                #field_name: std::option::Option<#field_type>
            }
        }
    })
}

/// Generate setter methods for builder fields
fn generate_setter_methods<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_attrs: &'a StructAttributes,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
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
                quote::quote! {
                    #(#field_doc_attrs)*
                    pub fn #field_name(mut self, value: impl std::convert::Into<#inner_type>) -> Self {
                        self.#field_name = std::option::Option::Some(value.into());
                        self
                    }
                }
            } else {
                quote::quote! {
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
                quote::quote! {
                    #(#field_doc_attrs)*
                    pub fn #field_name(mut self, value: impl std::convert::Into<#field_type>) -> Self {
                        self.#field_name = std::option::Option::Some(value.into());
                        self
                    }
                }
            } else {
                quote::quote! {
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
fn generate_field_processing<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_attrs: &'a StructAttributes,
) -> impl Iterator<Item = proc_macro2::TokenStream> + 'a {
    fields.iter().map(move |field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let field_name_str = field_name.as_ref().unwrap().to_string();
        let temp_var = quote::format_ident!("__field_{}", field_name.as_ref().unwrap());

        // Parse field-level attributes
        let field_attrs = parse_field_attributes(&field.attrs).unwrap_or_default();
        let use_default = struct_attrs.default || field_attrs.default;
        let validate_fn = field_attrs.validate.as_ref();

        // Generate value extraction
        let value_extraction = if option_inner_type(field_type).is_some() {
            // Field is already Option, just use it
            quote::quote! {
                let #temp_var = self.#field_name;
            }
        } else if use_default {
            // Field has default attribute, use Default::default() if not set
            quote::quote! {
                let #temp_var = self.#field_name.unwrap_or_default();
            }
        } else {
            // Field is required, error if not set
            quote::quote! {
                let #temp_var = self.#field_name.ok_or_else(|| format!("Missing required field: {}", #field_name_str))?;
            }
        };

        // Generate validation if needed
        let validation = if let Some(validate_fn) = validate_fn {
            if option_inner_type(field_type).is_some() {
                // Optional field: validate if Some
                quote::quote! {
                    if let std::option::Option::Some(ref value) = #temp_var {
                        #validate_fn(value).map_err(|e| format!("Validation failed for field '{}': {}", #field_name_str, e))?;
                    }
                }
            } else {
                // Non-optional field: always validate
                quote::quote! {
                    #validate_fn(&#temp_var).map_err(|e| format!("Validation failed for field '{}': {}", #field_name_str, e))?;
                }
            }
        } else {
            quote::quote! {}
        };

        quote::quote! {
            #value_extraction
            #validation
        }
    })
}

/// Generate final field assignments for struct construction
fn generate_build_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
    fields.iter().map(|field| {
        let field_name = &field.ident;
        let temp_var = quote::format_ident!("__field_{}", field_name.as_ref().unwrap());

        quote::quote! {
            #field_name: #temp_var
        }
    })
}

/// Generate send() and send_async() methods for the builder
fn generate_builder_send_methods(
    struct_name: &syn::Ident,
    struct_attrs: &StructAttributes,
) -> proc_macro2::TokenStream {
    let builder_name = quote::format_ident!("{}Builder", struct_name);
    let method_value = struct_attrs.method.as_ref().map(|s| s.as_str()).unwrap_or("GET");

    quote::quote! {
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
            pub fn send(mut self) -> std::result::Result<std::vec::Vec<u8>, std::string::String> {
                // Extract client and base URL before building
                let client = self.__http_client.take()
                    .ok_or_else(|| "No HTTP client configured. Use .http_client() to set one.".to_string())?;

                let base_url = self.__base_url.take()
                    .ok_or_else(|| "No base URL configured. Use .base_url() to set one.".to_string())?;

                let request = self.build()?;
                let path = request.build_url().map_err(|e| format!("Failed to build URL: {}", e))?;
                let url = format!("{}{}", base_url, path);
                let headers = request.build_headers();
                let body = request.build_body()?;

                client.send(#method_value, &url, headers, body)
                    .map_err(|e| format!("HTTP request failed: {:?}", e))
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
            pub async fn send_async(mut self) -> std::result::Result<std::vec::Vec<u8>, std::string::String> {
                // Extract client and base URL before building
                let client = self.__async_http_client.take()
                    .ok_or_else(|| "No async HTTP client configured. Use .async_http_client() to set one.".to_string())?;

                let base_url = self.__base_url.take()
                    .ok_or_else(|| "No base URL configured. Use .base_url() to set one.".to_string())?;

                let request = self.build()?;
                let path = request.build_url().map_err(|e| format!("Failed to build URL: {}", e))?;
                let url = format!("{}{}", base_url, path);
                let headers = request.build_headers();
                let body = request.build_body()?;

                client.send_async(#method_value, &url, headers, body).await
                    .map_err(|e| format!("HTTP request failed: {:?}", e))
            }
        }
    }
}

/// Generate the impl block with HTTP-related methods (build_url, build_body, build_headers, send_with_client)
fn generate_http_methods_impl(
    struct_name: &syn::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    struct_attrs: &StructAttributes,
) -> proc_macro2::TokenStream {
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
        let build_headers_method = generate_build_headers_method(&header_fields);
        let send_with_client_method = generate_send_with_client_method(struct_attrs);

        quote::quote! {
            impl #struct_name {
                #[doc = "Builds the URL path by substituting path parameters and appending query string."]
                #[doc = ""]
                #[doc = "# Errors"]
                #[doc = ""]
                #[doc = "Returns an error if any required path parameters are not set or if query serialization fails."]
                pub fn build_url(&self) -> std::result::Result<std::string::String, std::string::String> {
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
        quote::quote! {}
    }
}

/// Generate path parameter replacement code
fn generate_path_replacements(
    path_params: &[String],
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<proc_macro2::TokenStream> {
    path_params.iter().map(|param| {
        let matching_field = fields.iter().find(|field| {
            field.ident.as_ref().unwrap().to_string() == *param
        });

        if let Some(field) = matching_field {
            let field_name = &field.ident;
            let placeholder = format!("{{{}}}", param);
            let is_option = option_inner_type(&field.ty).is_some();

            if is_option {
                quote::quote! {
                    path = path.replace(#placeholder, &self.#field_name
                        .as_ref()
                        .ok_or_else(|| format!("Missing required path parameter: {}", #param))?
                        .to_string());
                }
            } else {
                quote::quote! {
                    path = path.replace(#placeholder, &self.#field_name.to_string());
                }
            }
        } else {
            quote::quote! {
                compile_error!(concat!("Path parameter '", #param, "' does not match any field"));
            }
        }
    }).collect()
}

/// Generate query string serialization code
fn generate_query_serialization(
    query_fields: &[&syn::Field],
    struct_attrs: &StructAttributes,
) -> proc_macro2::TokenStream {
    if query_fields.is_empty() {
        return quote::quote! {};
    }

    let query_struct_fields = query_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let serde_attrs = extract_serde_attributes(&field.attrs);

        let skip_attr = if option_inner_type(field_type).is_some() {
            quote::quote! { #[serde(skip_serializing_if = "Option::is_none")] }
        } else {
            quote::quote! {}
        };

        quote::quote! {
            #(#serde_attrs)*
            #skip_attr
            #field_name: #field_type
        }
    });

    let query_field_assignments = query_fields.iter().map(|field| {
        let field_name = &field.ident;
        quote::quote! { #field_name: self.#field_name.clone() }
    });

    let config_expr = if let Some(config) = &struct_attrs.query_config {
        let config_tokens: proc_macro2::TokenStream = config.parse().unwrap();
        quote::quote! { #config_tokens }
    } else {
        quote::quote! { serde_qs::Config::new() }
    };

    quote::quote! {
        #[derive(serde::Serialize)]
        struct QueryParams {
            #(#query_struct_fields),*
        }

        let query_params = QueryParams {
            #(#query_field_assignments),*
        };

        let config = #config_expr;
        let query_string = config.serialize_string(&query_params)
            .map_err(|e| format!("Failed to serialize query parameters: {}", e))?;

        if !query_string.is_empty() {
            path.push('?');
            path.push_str(&query_string);
        }
    }
}

/// Generate the build_body() method
fn generate_build_body_method(body_fields: &[&syn::Field]) -> proc_macro2::TokenStream {
    if body_fields.is_empty() {
        return quote::quote! {
            #[doc = "Builds the request body (always returns None as there are no body fields)."]
            pub fn build_body(&self) -> std::result::Result<std::option::Option<std::vec::Vec<u8>>, std::string::String> {
                std::result::Result::Ok(std::option::Option::None)
            }
        };
    }

    let body_struct_fields = body_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let serde_attrs = extract_serde_attributes(&field.attrs);

        let skip_attr = if option_inner_type(field_type).is_some() {
            quote::quote! { #[serde(skip_serializing_if = "Option::is_none")] }
        } else {
            quote::quote! {}
        };

        quote::quote! {
            #(#serde_attrs)*
            #skip_attr
            #field_name: #field_type
        }
    });

    let body_field_assignments = body_fields.iter().map(|field| {
        let field_name = &field.ident;
        quote::quote! { #field_name: self.#field_name.clone() }
    });

    quote::quote! {
        #[doc = "Builds the request body as JSON."]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an error if JSON serialization fails."]
        pub fn build_body(&self) -> std::result::Result<std::option::Option<std::vec::Vec<u8>>, std::string::String> {
            #[derive(serde::Serialize)]
            struct BodyParams {
                #(#body_struct_fields),*
            }

            let body_params = BodyParams {
                #(#body_field_assignments),*
            };

            let json = serde_json::to_vec(&body_params)
                .map_err(|e| format!("Failed to serialize body: {}", e))?;

            std::result::Result::Ok(std::option::Option::Some(json))
        }
    }
}

/// Generate the build_headers() method
fn generate_build_headers_method(header_fields: &[&syn::Field]) -> proc_macro2::TokenStream {
    if header_fields.is_empty() {
        return quote::quote! {
            #[doc = "Builds HTTP headers (empty as there are no header fields)."]
            pub fn build_headers(&self) -> std::collections::HashMap<std::string::String, std::string::String> {
                std::collections::HashMap::new()
            }
        };
    }

    let header_insertions = header_fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        let field_name_str = field_name.as_ref().unwrap().to_string();

        let field_attrs = parse_field_attributes(&field.attrs).unwrap_or_default();
        let header_name = field_attrs.rename
            .unwrap_or_else(|| snake_to_title_case(&field_name_str));

        if option_inner_type(field_type).is_some() {
            quote::quote! {
                if let std::option::Option::Some(ref value) = self.#field_name {
                    headers.insert(#header_name.to_string(), value.to_string());
                }
            }
        } else {
            quote::quote! {
                headers.insert(#header_name.to_string(), self.#field_name.to_string());
            }
        }
    });

    quote::quote! {
        #[doc = "Builds HTTP headers from header-annotated fields."]
        pub fn build_headers(&self) -> std::collections::HashMap<std::string::String, std::string::String> {
            let mut headers = std::collections::HashMap::new();
            #(#header_insertions)*
            headers
        }
    }
}

/// Generate the send_with_client() method
fn generate_send_with_client_method(struct_attrs: &StructAttributes) -> proc_macro2::TokenStream {
    let method_value = struct_attrs.method.as_ref().map(|s| s.as_str()).unwrap_or("GET");

    quote::quote! {
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
        ) -> std::result::Result<std::vec::Vec<u8>, std::string::String> {
            let path = self.build_url().map_err(|e| format!("Failed to build URL: {}", e))?;
            let url = format!("{}{}", base_url, path);
            let headers = self.build_headers();
            let body = self.build_body()?;

            client.send(#method_value, &url, headers, body)
                .map_err(|e| format!("HTTP request failed: {:?}", e))
        }
    }
}

// Based on https://duskmoon314.com/en/blog/2022/10/01/extract-type-from-option-in-rs-procmacro/
fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
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

/// Parse struct-level #[request_builder(...)] attributes
fn parse_struct_attributes(attrs: &[syn::Attribute]) -> syn::Result<StructAttributes> {
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
fn parse_field_attributes(attrs: &[syn::Attribute]) -> syn::Result<FieldAttributes> {
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

/// Extract doc comments and other documentation attributes to copy to generated code
fn extract_doc_attributes(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| {
            // Keep doc comments and other documentation attributes
            attr.path().is_ident("doc")
        })
        .cloned()
        .collect()
}

/// Extract path parameters from a URL path template
/// For example, "/api/users/{id}/posts/{post_id}" -> ["id", "post_id"]
fn extract_path_params(path: &str) -> Vec<String> {
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

/// Extract serde attributes to copy to generated structs
fn extract_serde_attributes(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| {
            // Keep serde attributes
            attr.path().is_ident("serde")
        })
        .cloned()
        .collect()
}

/// Convert snake_case to Title-Case for HTTP headers
/// For example: "authorization" -> "Authorization", "content_type" -> "Content-Type"
fn snake_to_title_case(s: &str) -> String {
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
