use super::MacroCall;
use crate::{
    parser::{
        ParamKind,
        FreshAttributeParser,
        MethodMetaParser,
        Parser
    }
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemTrait, TraitItem};

pub struct FreshExpander;

impl super::Expander for FreshExpander {
    fn expand(&self, call: MacroCall) -> syn::Result<TokenStream> {
        match call.form {
            super::MacroForm::Attribute { attr, item } => {
                // 解析宏属性
                let attributes = FreshAttributeParser::parse(&attr)?;

                // 展开宏
                let mut trait_item: ItemTrait = syn::parse2(item.clone())?;

                let trait_ident = trait_item.ident.clone();
                let client_ident = format_ident!("{}Client", trait_ident);

                // 收集方法元信息（剥离前）
                let methods = MethodMetaParser::parse(&trait_item)?;

                // 剥离自定义宏 避免“未知属性”错误
                strip_custom_attrs_in_trait(&mut trait_item);

                // 展开每个方法
                let mut method_impls = Vec::new();
                for m in &methods {
                    method_impls.push(expand_method_impl(&m));
                }

                // 构造函数
                let mut ctor_extra = quote! {
                    pub fn with_endpoint(endpoint: &str) -> ::fresh::Result<Self> {
                        Ok(Self { core: ::fresh::HttpClient::with_endpoint(endpoint)? })
                    }
                };

                // 生成 .endpoint(...) 语句
                let endpoint_stmt = if let Some(endpoint) = &attributes.endpoint {
                    quote! { .endpoint(#endpoint) }
                } else {
                    quote! {}
                };

                // 生成 .headers(vec![...])
                let headers_pairs = attributes
                    .headers
                    .into_iter()
                    .map(|(k, v)| {
                        // "_" -> "-"
                        let k = k.replace('_', "-").to_ascii_lowercase();
                        let k = syn::LitStr::new(&k, proc_macro2::Span::call_site());
                        let v = syn::LitStr::new(&v, proc_macro2::Span::call_site());
                        quote! { (::std::string::String::from(#k), ::std::string::String::from(#v)) }
                    })
                    .collect::<Vec<_>>();

                let headers_stmt = if headers_pairs.is_empty() {
                    quote! {}
                } else {
                    quote! { .headers(vec![#(#headers_pairs),*]) }
                };

                // 生成 timeout 语句
                let timeout_stmt = if let Some(timeout_ms) = attributes.timeout {
                    let timeout_duration =
                        quote! { ::std::time::Duration::from_millis(#timeout_ms) };
                    quote! { .timeout(#timeout_duration) }
                } else {
                    quote! {}
                };

                // 生成 connect_timeout 语句
                let connect_timeout_stmt =
                    if let Some(connect_timeout_ms) = attributes.connect_timeout {
                        let connect_timeout_duration =
                            quote! { ::std::time::Duration::from_millis(#connect_timeout_ms) };
                        quote! { .connect_timeout(#connect_timeout_duration) }
                    } else {
                        quote! {}
                    };

                // 生成 read_timeout 语句
                let read_timeout_stmt = if let Some(read_timeout_ms) = attributes.read_timeout
                {
                    let read_timeout_duration =
                        quote! { ::std::time::Duration::from_millis(#read_timeout_ms) };
                    quote! { .read_timeout(#read_timeout_duration) }
                } else {
                    quote! {}
                };

                // 附加 new_default 构造函数
                ctor_extra = quote! {
                    #ctor_extra
                    pub fn new_default() -> ::fresh::Result<Self> {
                        let option = ::fresh::HttpClientOption::builder()
                            #endpoint_stmt
                            #headers_stmt
                            #timeout_stmt
                            #connect_timeout_stmt
                            #read_timeout_stmt
                            .build()
                            .map_err(|e| ::fresh::Error::InvalidArgument(format!("Build HttpClientOption failed: {}", e)))?;

                        Ok(Self { core: ::fresh::HttpClient::new(option)? })
                    }
                };

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

                Ok(expanded)
            }
            _ => Err(syn::Error::new_spanned(
                TokenStream::new(),
                "Unsupported macro form for FreshExpander",
            )),
        }
    }
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

fn expand_method_impl(m: &crate::parser::MethodMeta) -> TokenStream {
    let ident = &m.sig_ident; // Ident 实现 ToTokens，可以直接插值
    let ok_ty = &m.ok_ty; // 已是 TokenStream，可以直接插值
    let route = &m.route;
    // 如果HTTP 方法为空则报错
    let Some(method) = route.method.as_ref() else {
        // 注意：compile_error! 只接受字面量，不能用 "{}", 用 format! 先生成字面量，或用 concat! + stringify!
        let msg = format!("HTTP method not specified for method {}", ident); // ident: syn::Ident
        return quote::quote! {
            compile_error!(#msg);
        };
    };

    let method_tokens = method.to_token();

    // impl 参数签名（移除参数属性）
    let mut impl_params = Vec::new();
    for p in &m.params {
        if let Some(ty) = &p.ty {
            let id = &p.ident;
            impl_params.push(quote! { #id: #ty });
        }
    }

    // path 占位符替换
    let Some(path) = route.path.as_ref() else {
        let msg = format!("Path not specified for method {}", ident);
        return quote::quote! {
            compile_error!(#msg);
        };
    };
    let path_lit = quote! { #path }; // 先绑定要插值的字段
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
