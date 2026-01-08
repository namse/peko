use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{env, fs, path::Path};

pub fn generate_routes() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let pages_dir = Path::new(&manifest_dir).join("src/pages");
    let output_path = Path::new(&manifest_dir).join("src/route_generated.rs");

    println!("cargo:rerun-if-changed=src/pages");

    let pages = discover_pages(&pages_dir);
    let tokens = generate_code(&pages);

    let syntax_tree = syn::parse2::<syn::File>(tokens).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    fs::write(&output_path, formatted).unwrap();
}

#[derive(Debug)]
struct PageInfo {
    module_name: String,
    module_path: String,
    route_path: String,
    route_segments: Vec<RouteSegment>,
    path_params: Option<Vec<PathParamField>>,
    search_params: Option<Vec<SearchParamField>>,
}

#[derive(Debug, Clone)]
enum RouteSegment {
    Static(String),
    Dynamic(String), // [id] -> "id"
}

#[derive(Debug)]
struct SearchParamField {
    name: String,
    is_optional: bool,
    inner_type: String,
}

#[derive(Debug)]
struct PathParamField {
    name: String,
    inner_type: String,
}

fn parse_search_params(content: &str) -> Option<Vec<SearchParamField>> {
    let syntax_tree = syn::parse_file(content).ok()?;

    for item in syntax_tree.items {
        if let syn::Item::Struct(item_struct) = item
            && item_struct.ident == "SearchParams"
        {
            let mut fields = Vec::new();

            if let syn::Fields::Named(named_fields) = item_struct.fields {
                for field in named_fields.named {
                    let name = field.ident?.to_string();
                    let (is_optional, inner_type) = extract_type_info(&field.ty);
                    fields.push(SearchParamField {
                        name,
                        is_optional,
                        inner_type,
                    });
                }
            }

            return Some(fields);
        }
    }

    None
}

fn parse_path_params(content: &str) -> Option<Vec<PathParamField>> {
    let syntax_tree = syn::parse_file(content).ok()?;

    for item in syntax_tree.items {
        if let syn::Item::Struct(item_struct) = item
            && item_struct.ident == "PathParams"
        {
            let mut fields = Vec::new();

            if let syn::Fields::Named(named_fields) = item_struct.fields {
                for field in named_fields.named {
                    let name = field.ident?.to_string();
                    let (_is_optional, inner_type) = extract_type_info(&field.ty);
                    fields.push(PathParamField { name, inner_type });
                }
            }

            return Some(fields);
        }
    }

    None
}

fn extract_type_info(ty: &syn::Type) -> (bool, String) {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        if segment.ident == "Option"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
        {
            return (true, quote!(#inner_ty).to_string());
        }
        return (false, quote!(#ty).to_string());
    }
    (false, quote!(#ty).to_string())
}

fn discover_pages(pages_dir: &Path) -> Vec<PageInfo> {
    let mut pages = Vec::new();

    if !pages_dir.exists() {
        return pages;
    }

    discover_pages_recursive(pages_dir, pages_dir, &mut pages);
    pages
}

fn discover_pages_recursive(base_dir: &Path, current_dir: &Path, pages: &mut Vec<PageInfo>) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            let mod_rs = path.join("mod.rs");
            if mod_rs.exists() {
                let relative_path = path.strip_prefix(base_dir).unwrap();
                let path_segments: Vec<_> = relative_path
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect();

                // module_name: [id] -> _id_ for valid identifier
                let module_name = format!(
                    "pages_{}",
                    path_segments
                        .iter()
                        .map(|s| {
                            if s.starts_with('[') && s.ends_with(']') {
                                format!("_{}_", &s[1..s.len() - 1])
                            } else {
                                s.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("_")
                );
                let module_path = format!("pages/{}/mod.rs", path_segments.join("/"));
                let route_path = if path_segments == vec!["index"] {
                    "/".to_string()
                } else {
                    format!("/{}", path_segments.join("/"))
                };

                // Parse route segments for pattern matching
                let route_segments: Vec<RouteSegment> = path_segments
                    .iter()
                    .map(|s| {
                        if s.starts_with('[') && s.ends_with(']') {
                            RouteSegment::Dynamic(s[1..s.len() - 1].to_string())
                        } else {
                            RouteSegment::Static(s.clone())
                        }
                    })
                    .collect();

                let content = fs::read_to_string(&mod_rs).ok();
                let search_params = content
                    .as_ref()
                    .and_then(|c| parse_search_params(c));
                let path_params = content
                    .as_ref()
                    .and_then(|c| parse_path_params(c));

                pages.push(PageInfo {
                    module_name,
                    module_path,
                    route_path,
                    route_segments,
                    path_params,
                    search_params,
                });
            }

            discover_pages_recursive(base_dir, &path, pages);
        }
    }
}

fn generate_code(pages: &[PageInfo]) -> TokenStream {
    let module_declarations = generate_module_declarations(pages);
    let route_matches = generate_route_matches(pages);

    let route_chain = if route_matches.is_empty() {
        quote! {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap())
        }
    } else {
        let first = &route_matches[0];
        let rest = &route_matches[1..];
        quote! {
            #first
            #(else #rest)*
            else {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap())
            }
        }
    };

    quote! {
        // Auto-generated by build.rs

        #(#module_declarations)*

        use forte_sdk::anyhow::Result;
        use forte_sdk::wstd::http::{Error, Request, Response, StatusCode, body::Body, HeaderMap};
        use forte_sdk::http::header::COOKIE;
        use forte_sdk::*;
        use std::collections::HashMap;

        #[forte_sdk::wstd::http_server]
        pub async fn main(request: Request<Body>) -> Result<Response<Body>, Error> {
            let (parts, _body) = request.into_parts();
            let headers = parts.headers;
            let path = parts.uri.path();
            let query = parts.uri.query().unwrap_or("");
            let cookie_jar = make_cookie_jar(&headers);
            let query_params: HashMap<String, String> = query
                .split('&')
                .filter(|s| !s.is_empty())
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next()?;
                    let value = parts.next().unwrap_or("");
                    Some((key.to_string(), value.to_string()))
                })
                .collect();

            #route_chain
        }

        fn make_cookie_jar(headers: &HeaderMap) -> cookie::CookieJar {
            let mut jar = cookie::CookieJar::new();
            let Some(cookie) = headers.get(COOKIE) else {
                return jar;
            };
            let Ok(cookie_str) = cookie.to_str() else {
                return jar;
            };

            for cookie in cookie::Cookie::split_parse_encoded(cookie_str) {
                let Ok(cookie) = cookie else { continue };
                jar.add_original(cookie.into_owned());
            }

            jar
        }
    }
}

fn generate_module_declarations(pages: &[PageInfo]) -> Vec<TokenStream> {
    pages
        .iter()
        .map(|page| {
            let module_name = format_ident!("{}", page.module_name);
            let module_path = &page.module_path;

            quote! {
                #[path = #module_path]
                mod #module_name;
            }
        })
        .collect()
}

fn has_dynamic_segments(segments: &[RouteSegment]) -> bool {
    segments
        .iter()
        .any(|s| matches!(s, RouteSegment::Dynamic(_)))
}

fn generate_route_matches(pages: &[PageInfo]) -> Vec<TokenStream> {
    pages
        .iter()
        .map(|page| {
            let module_name = format_ident!("{}", page.module_name);

            // Generate route condition
            let route_condition = if has_dynamic_segments(&page.route_segments) {
                generate_dynamic_route_condition(&page.route_segments)
            } else {
                let route_path = &page.route_path;
                quote! { path == #route_path }
            };

            // Generate path params extraction (if dynamic)
            let path_params_extraction = if page.path_params.is_some() {
                generate_path_params_extraction(&module_name, &page.route_segments, page.path_params.as_ref().unwrap())
            } else {
                quote! {}
            };

            // Generate search params extraction
            let search_params_extraction = if let Some(fields) = &page.search_params {
                let field_parsers = generate_search_field_parsers(fields);
                let field_names: Vec<_> =
                    fields.iter().map(|f| format_ident!("{}", f.name)).collect();
                quote! {
                    #(#field_parsers)*
                    let search_params = #module_name::SearchParams {
                        #(#field_names),*
                    };
                }
            } else {
                quote! {}
            };

            // Generate handler call based on what params exist
            let handler_call = match (&page.path_params, &page.search_params) {
                (Some(_), Some(_)) => quote! {
                    #module_name::handler(headers, cookie_jar, path_params, search_params).await
                },
                (Some(_), None) => quote! {
                    #module_name::handler(headers, cookie_jar, path_params).await
                },
                (None, Some(_)) => quote! {
                    #module_name::handler(headers, cookie_jar, search_params).await
                },
                (None, None) => quote! {
                    #module_name::handler(headers, cookie_jar).await
                },
            };

            quote! {
                if #route_condition {
                    #path_params_extraction
                    #search_params_extraction

                    match #handler_call {
                        Ok(props) => {
                            let stream = forte_json::to_stream(&props);
                            Ok(Response::new(Body::from_stream(stream)))
                        }
                        Err(e) => {
                            Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from(format!("Error: {:?}", e)))
                                .unwrap())
                        }
                    }
                }
            }
        })
        .collect()
}

fn generate_dynamic_route_condition(segments: &[RouteSegment]) -> TokenStream {
    let segment_count = segments.len();
    let segment_checks: Vec<TokenStream> = segments
        .iter()
        .enumerate()
        .filter_map(|(i, seg)| {
            if let RouteSegment::Static(s) = seg {
                Some(quote! { path_segments.get(#i) == Some(&#s) })
            } else {
                None // Dynamic segments match anything
            }
        })
        .collect();

    if segment_checks.is_empty() {
        quote! {
            {
                let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
                path_segments.len() == #segment_count
            }
        }
    } else {
        quote! {
            {
                let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
                path_segments.len() == #segment_count && #(#segment_checks)&&*
            }
        }
    }
}

fn generate_path_params_extraction(
    module_name: &syn::Ident,
    route_segments: &[RouteSegment],
    path_params: &[PathParamField],
) -> TokenStream {
    let extractions: Vec<TokenStream> = route_segments
        .iter()
        .enumerate()
        .filter_map(|(i, seg)| {
            if let RouteSegment::Dynamic(param_name) = seg {
                // Find the corresponding PathParamField
                let field = path_params.iter().find(|f| &f.name == param_name)?;
                let field_ident = format_ident!("{}", field.name);
                let inner_type: TokenStream = field.inner_type.parse().unwrap();

                if field.inner_type == "String" {
                    Some(quote! {
                        let #field_ident: String = path_segments[#i].to_string();
                    })
                } else {
                    Some(quote! {
                        let #field_ident: #inner_type = match path_segments[#i].parse::<#inner_type>() {
                            Ok(v) => v,
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!("Invalid path parameter: {}", stringify!(#field_ident))))
                                    .unwrap());
                            }
                        };
                    })
                }
            } else {
                None
            }
        })
        .collect();

    let field_names: Vec<_> = path_params
        .iter()
        .map(|f| format_ident!("{}", f.name))
        .collect();

    quote! {
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        #(#extractions)*
        let path_params = #module_name::PathParams { #(#field_names),* };
    }
}

fn generate_search_field_parsers(fields: &[SearchParamField]) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_name = format_ident!("{}", field.name);
            let field_name_str = &field.name;
            let inner_type: TokenStream = field.inner_type.parse().unwrap();

            if field.is_optional {
                if field.inner_type == "String" {
                    quote! {
                        let #field_name: Option<String> = query_params.get(#field_name_str).cloned();
                    }
                } else {
                    quote! {
                        let #field_name: Option<#inner_type> = query_params
                            .get(#field_name_str)
                            .and_then(|v| v.parse::<#inner_type>().ok());
                    }
                }
            } else if field.inner_type == "String" {
                quote! {
                    let Some(#field_name) = query_params.get(#field_name_str).cloned() else {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("Missing required query parameter: {}", #field_name_str)))
                            .unwrap());
                    };
                }
            } else {
                quote! {
                    let #field_name: #inner_type = match query_params.get(#field_name_str) {
                        Some(v) => match v.parse::<#inner_type>() {
                            Ok(parsed) => parsed,
                            Err(_) => {
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(Body::from(format!("Invalid value for query parameter: {}", #field_name_str)))
                                    .unwrap());
                            }
                        },
                        None => {
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Missing required query parameter: {}", #field_name_str)))
                                .unwrap());
                        }
                    };
                }
            }
        })
        .collect()
}

