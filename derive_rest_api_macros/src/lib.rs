
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
    /// Response type
    response: Option<syn::Type>,
    /// Query string config expression (e.g., "my_qs_config()")
    query_config: Option<String>,
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
}

#[proc_macro_derive(RequestBuilder, attributes(request_builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match generate_request_builder(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_request_builder(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
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
    // If field is already Option<T>, keep it as Option<T>
    // Otherwise, wrap it in Option
    let builder_fields = fields.iter().map(|field| {
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
    });

    // Generate field names for constructor
    let field_names = fields.iter().map(|field| &field.ident);

    // Generate setter methods for each field
    let setter_methods = fields.iter().map(|field| {
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
    });

    // Generate field extraction and validation for build() method
    // We need to extract values, then validate them, then construct the struct
    let field_processing = fields.iter().map(|field| {
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
    });

    // Generate final field assignments using temporary variables
    let build_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let temp_var = quote::format_ident!("__field_{}", field_name.as_ref().unwrap());

        quote::quote! {
            #field_name: #temp_var
        }
    });

    // Generate build_url() impl for the original struct if path attribute is present
    let build_url_impl = if let Some(path_template) = &struct_attrs.path {
        // Find all path parameters in the template (e.g., {id}, {user_id})
        let path_params: Vec<String> = extract_path_params(path_template);

        // Collect query fields (fields with #[request_builder(query)])
        let query_fields: Vec<_> = fields.iter().filter(|field| {
            if let Ok(attrs) = parse_field_attributes(&field.attrs) {
                attrs.kind == FieldKind::Query
            } else {
                false
            }
        }).collect();

        // Generate code to replace each path parameter with the field value
        let path_replacements = path_params.iter().map(|param| {
            // Find the field that matches this path parameter
            let matching_field = fields.iter().find(|field| {
                let field_name = field.ident.as_ref().unwrap().to_string();
                field_name == *param
            });

            if let Some(field) = matching_field {
                let field_name = &field.ident;
                let placeholder = format!("{{{}}}", param);

                // For the original struct, fields are direct values, not Option
                // Check if the field is Option type to determine how to access it
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
                // Path parameter doesn't match any field - generate a compile error
                quote::quote! {
                    compile_error!(concat!("Path parameter '", #param, "' does not match any field"));
                }
            }
        });

        // Generate query string serialization if there are query fields
        let query_serialization = if !query_fields.is_empty() {
            // Generate struct definition for query parameters
            let query_struct_fields = query_fields.iter().map(|field| {
                let field_name = &field.ident;
                let field_type = &field.ty;
                quote::quote! {
                    #[serde(skip_serializing_if = "Option::is_none")]
                    #field_name: #field_type
                }
            });

            // Generate field assignments from self
            // For the original struct, we reference the fields directly
            let query_field_assignments = query_fields.iter().map(|field| {
                let field_name = &field.ident;
                let field_type = &field.ty;

                // Check if it needs to be cloned (for non-Copy types like String)
                // We'll use Clone for all query fields to be safe
                if option_inner_type(field_type).is_some() {
                    // Option type - clone the option
                    quote::quote! {
                        #field_name: self.#field_name.clone()
                    }
                } else {
                    // Non-option but needs to be wrapped in Option for serialization
                    quote::quote! {
                        #field_name: std::option::Option::Some(self.#field_name.clone())
                    }
                }
            });

            // Determine config expression (custom or default)
            let config_expr = if let Some(config) = &struct_attrs.query_config {
                let config_tokens: proc_macro2::TokenStream = config.parse().unwrap();
                quote::quote! { #config_tokens }
            } else {
                quote::quote! { serde_qs::Config::new() }
            };

            quote::quote! {
                // Create a temporary struct to hold query parameters
                #[derive(serde::Serialize)]
                struct QueryParams {
                    #(#query_struct_fields),*
                }

                let query_params = QueryParams {
                    #(#query_field_assignments),*
                };

                // Serialize query parameters
                let config = #config_expr;
                let query_string = config.serialize_string(&query_params)
                    .map_err(|e| format!("Failed to serialize query parameters: {}", e))?;

                if !query_string.is_empty() {
                    path.push('?');
                    path.push_str(&query_string);
                }
            }
        } else {
            quote::quote! {}
        };

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
            }
        }
    } else {
        quote::quote! {}
    };

    // Generate the builder struct and its impl block
    let expanded = quote::quote! {
        #(#struct_doc_attrs)*
        #[doc = ""]
        #[doc = concat!("Builder for [`", stringify!(#struct_name), "`].")]
        pub struct #builder_name {
            #(#builder_fields),*
        }

        impl #builder_name {
            #[doc = concat!("Creates a new [`", stringify!(#builder_name), "`] with all fields set to `None`.")]
            pub fn new() -> Self {
                Self {
                    #(#field_names: std::option::Option::None),*
                }
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

        // Generate build_url() impl for the original struct
        #build_url_impl
    };

    Ok(expanded)
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

            // #[request_builder(query)]
            if meta.path.is_ident("query") {
                result.kind = FieldKind::Query;
                return Ok(());
            }

            // #[request_builder(body)]
            if meta.path.is_ident("body") {
                result.kind = FieldKind::Body;
                return Ok(());
            }

            // #[request_builder(header)]
            if meta.path.is_ident("header") {
                result.kind = FieldKind::Header;
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
