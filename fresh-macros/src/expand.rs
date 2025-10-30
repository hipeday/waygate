//! 代码生成主流程（syn v2 兼容版）。
//!
//! 流程：
//! 1) 解析 `#[fresh(...)]` 的 trait 级参数（目前支持 base_url）
//! 2) 收集方法/参数注解元信息
//! 3) 剥离自定义注解，输出“干净的 trait”
//! 4) 生成 `XxxClient` 结构与构造函数（with_base_url/new_default）
//! 5) 为每个方法展开基于 reqwest 的实际调用代码

use quote::{format_ident, quote};
use syn::{FnArg, ItemTrait, TraitItem};

use crate::method::parse_method_attr;
use crate::param::{ParamKind, parse_param_attrs};
use crate::parser::Parser;
use crate::util::{extract_ok_type, http_method_tokens};

/// 使用 proc_macro2::TokenStream，入口在 lib.rs 里做类型转换
pub fn expand_fresh(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    // 解析 trait 级参数
    let fresh_attributes = match crate::parser::FreshAttributeParser::parse(&attr) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    // 解析 trait 本体
    let mut trait_item: ItemTrait = match syn::parse2(item) {
        Ok(x) => x,
        Err(e) => return e.to_compile_error(),
    };

    let trait_ident = trait_item.ident.clone();
    let client_ident = format_ident!("{}Client", trait_ident);

    // 收集方法元信息（剥离前）
    let methods = collect_methods(&trait_item);

    // 剥离自定义注解，避免“未知属性”错误
    strip_custom_attrs_in_trait(&mut trait_item);

    // 展开每个方法
    let mut method_impls = Vec::new();
    for m in methods {
        method_impls.push(expand_method_impl(&m));
    }

    // 构造函数
    let mut ctor_extra = quote! {
        pub fn with_endpoint(endpoint: &str) -> ::fresh::Result<Self> {
            Ok(Self { core: ::fresh::HttpClient::with_endpoint(endpoint)? })
        }
    };

    let headers_pairs = fresh_attributes
        .headers
        .into_iter()
        .map(|(k, v)| {
            // "_" -> "-"
            let k = k.replace('_', "-");
            let k = syn::LitStr::new(&k, proc_macro2::Span::call_site());
            let v = syn::LitStr::new(&v, proc_macro2::Span::call_site());
            quote! { (#k, #v) }
        })
        .collect::<Vec<_>>();

    // 2) 只有在非空时才生成 .headers(vec![...])
    let headers_stmt = if headers_pairs.is_empty() {
        quote! {}
    } else {
        quote! { .headers(vec![#(#headers_pairs),*]) }
    };

    if let Some(endpoint) = fresh_attributes.endpoint {
        ctor_extra = quote! {
            #ctor_extra
            pub fn new_default() -> ::fresh::Result<Self> {
                let option = ::fresh::HttpClientOption::builder()
                    .endpoint(#endpoint)
                    #headers_stmt
                    .build()?;

                Ok(Self { core: ::fresh::HttpClient::new(option)? })
            }
        };
    }

    let expanded = quote! {
        #trait_item

        pub struct #client_ident {
            pub core: ::fresh::HttpClient,
        }

        impl #client_ident {
            pub fn new(core: ::fresh::HttpClient) -> Self { Self { core } }
            #ctor_extra
        }

        impl #trait_ident for #client_ident {
            #(#method_impls)*
        }
    };

    expanded
}

struct MethodMeta {
    sig_ident: syn::Ident,
    ok_ty: proc_macro2::TokenStream,
    http_method: String,
    path_lit: proc_macro2::TokenStream,
    params: Vec<crate::param::ParamMeta>,
}

fn collect_methods(trait_item: &ItemTrait) -> Vec<MethodMeta> {
    let mut out = Vec::new();
    for it in &trait_item.items {
        let TraitItem::Fn(m) = it else {
            continue;
        };
        let sig_ident = m.sig.ident.clone();

        let (http_method, path_lit) =
            // 英文异常提示
            parse_method_attr(&m.attrs).unwrap_or_else(|| panic!("Method {} is missing #[get|post|put|delete|patch(\"/path\")] attribute", sig_ident));

        let ok_ty = extract_ok_type(&m.sig.output)
            .unwrap_or_else(|| panic!("Method {} must have return type Result<T, E>", sig_ident));

        let params = parse_param_attrs(&m.sig.inputs);

        out.push(MethodMeta {
            sig_ident,
            ok_ty,
            http_method,
            path_lit,
            params,
        });
    }
    out
}

fn strip_custom_attrs_in_trait(trait_item: &mut ItemTrait) {
    for item in &mut trait_item.items {
        if let TraitItem::Fn(m) = item {
            // 方法级：去掉 get/post/put/delete/patch
            m.attrs.retain(|a| {
                let Some(id) = a.path().get_ident() else {
                    return true;
                };
                let n = id.to_string();
                !matches!(n.as_str(), "get" | "post" | "put" | "delete" | "patch")
            });
            // 参数级：去掉 path/query/json/header
            for input in &mut m.sig.inputs {
                if let FnArg::Typed(pt) = input {
                    pt.attrs.retain(|a| {
                        let Some(id) = a.path().get_ident() else {
                            return true;
                        };
                        let n = id.to_string();
                        !matches!(n.as_str(), "path" | "query" | "json" | "header")
                    });
                }
            }
        }
    }
}

fn expand_method_impl(m: &MethodMeta) -> proc_macro2::TokenStream {
    let ident = &m.sig_ident; // Ident 实现 ToTokens，可以直接插值
    let ok_ty = &m.ok_ty; // 已是 TokenStream，可以直接插值
    let method_tokens = http_method_tokens(&m.http_method);

    // impl 参数签名（移除参数属性）
    let mut impl_params = Vec::new();
    for p in &m.params {
        if let Some(ty) = &p.ty {
            let id = &p.ident;
            impl_params.push(quote! { #id: #ty });
        }
    }

    // path 占位符替换
    let path_lit = &m.path_lit; // 先绑定要插值的字段
    let mut path_build = quote! { let mut __path = #path_lit.to_string(); };
    for p in &m.params {
        if matches!(p.kind, ParamKind::Path) {
            let id = &p.ident;
            path_build = quote! {
                #path_build
                let __ph = format!("{{{}}}", stringify!(#id));
                __path = __path.replace(&__ph, &::std::string::ToString::to_string(&#id));
            };
        }
    }

    // 构建请求（依次链式追加）
    let mut req_chain = quote! {
        let __url = self.core.endpoint().join(&__path)?;
        let __req = self.core.client().request(#method_tokens, __url)
    };

    for p in &m.params {
        match &p.kind {
            ParamKind::Query => {
                let id = &p.ident;
                req_chain = quote! { #req_chain .query(&#id) };
            }
            ParamKind::Header(name) => {
                let id = &p.ident;
                let name_lit = name;
                req_chain = quote! { #req_chain .header(#name_lit, &::std::string::ToString::to_string(&#id)) };
            }
            ParamKind::Json => {
                let id = &p.ident;
                req_chain = quote! { #req_chain .json(&#id) };
            }
            _ => {}
        }
    }

    quote! {
        async fn #ident(&self, #(#impl_params),*) -> ::fresh::Result<#ok_ty> {
            #path_build
            #req_chain;
            let __resp = __req.send().await?.error_for_status()?;
            let __out = __resp.json::<#ok_ty>().await?;
            Ok(__out)
        }
    }
}
