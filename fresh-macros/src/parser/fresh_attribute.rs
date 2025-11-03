use crate::{
    http::method::Method,
    util::extract_ok_type,
};
use derive_builder::Builder;
use proc_macro2::TokenStream;
use std::str::FromStr;
use syn::{
    Attribute,
    ItemTrait,
    LitInt,
    LitStr,
    TraitItem,
    parse::Parser,
    spanned::Spanned,
    FnArg,
    Pat,
    punctuated::Punctuated,
    token::Comma
};

// 接口级宏上的属性解析器
pub struct FreshAttributeParser;

// 路由级宏上的属性解析器
pub struct FreshRouteAttributeParser;

// 方法元信息解析器
pub struct MethodMetaParser;

pub struct ParamMetaParser;

/// 接口级解析属性
#[derive(Debug)]
pub struct FreshAttributes {
    pub endpoint: Option<String>,       // 基础端点 URL
    pub headers: Vec<(String, String)>, // 额外请求头
    pub timeout: Option<u64>,           // 请求超时，单位毫秒
    pub connect_timeout: Option<u64>,   // 连接超时，单位毫秒
    pub read_timeout: Option<u64>,      // 读取超时，单位毫秒
}

#[derive(Debug)]
pub struct FreshRouteAttributes {
    pub method: Option<Method>,         // HTTP 方法
    pub path: Option<String>,           // 请求路径
    pub headers: Vec<(String, String)>, // 额外请求头
    pub timeout: Option<u64>,           // 请求超时，单位毫秒
    pub connect_timeout: Option<u64>,   // 连接超时，单位毫秒
    pub read_timeout: Option<u64>,      // 读取超时，单位毫秒
}

#[derive(Debug)]
pub struct MethodMeta {
    pub sig_ident: syn::Ident,
    pub ok_ty: TokenStream,
    pub params: Vec<ParamMeta>,
    pub route: FreshRouteAttributes,
}

/// 参数标注类型
#[derive(Clone, Debug)]
pub enum ParamKind {
    Path,
    Query,
    Json,
    Header(String),
    Other,
}

/// 参数标注元信息
#[derive(Clone, Debug)]
pub struct ParamMeta {
    // 参数名
    pub ident: syn::Ident,
    // 参数类型（可能为空，例如 `self` 参数）
    pub ty: Option<syn::Type>,
    // 标注类型
    pub kind: ParamKind,
}

#[derive(Default, Builder, Debug)]
struct AttributeProperties {
    #[builder(default = "Some(Method::Get)")]
    method: Option<Method>, // HTTP 方法
    #[builder(default = "Some(String::from(\"http://localhost\"))")]
    endpoint: Option<String>, // 基础端点 URL
    #[builder(default = "None")]
    path: Option<String>, // 请求路径
    #[builder(default)]
    headers: Vec<(String, String)>, // 额外请求头
    #[builder(default = "None")]
    timeout: Option<u64>, // 请求超时，单位毫秒
    #[builder(default = "None")]
    connect_timeout: Option<u64>, // 连接超时，单位毫秒
    #[builder(default = "None")]
    read_timeout: Option<u64>, // 读取超时，单位毫秒
}

impl FreshRouteAttributes {
    fn set_path_if_none(&mut self, path: String) {
        if self.path.is_none() {
            self.path = Some(path);
        }
    }
}

impl AttributeProperties {
    pub fn builder() -> AttributePropertiesBuilder {
        AttributePropertiesBuilder::default()
    }
}

impl crate::parser::Parser<TokenStream> for FreshAttributeParser {
    type Output = FreshAttributes;

    fn parse(input: &TokenStream) -> syn::Result<Self::Output> {
        let mut builder = AttributeProperties::builder();

        let parser = get_parser(&mut builder);

        parser.parse2(input.clone())?;

        let properties = builder.build().map_err(|e| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to build FreshAttributes: {}", e),
            )
        })?;
        Ok(FreshAttributes {
            endpoint: properties.endpoint,
            headers: properties.headers,
            timeout: properties.timeout,
            connect_timeout: properties.connect_timeout,
            read_timeout: properties.read_timeout,
        })
    }
}

impl crate::parser::Parser<Vec<Attribute>> for FreshRouteAttributeParser {
    type Output = Option<FreshRouteAttributes>;

    fn parse(attrs: &Vec<Attribute>) -> syn::Result<Self::Output> {
        let mut builder = AttributeProperties::builder();
        for attr in attrs {
            if let Some(ident) = attr.path().get_ident() {
                let name = ident.to_string().to_uppercase(); // 方法级注解名称
                if !matches!(
                    name.as_str(),
                    "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" | "TRACE"
                ) {
                    continue;
                }

                builder.method(Some(
                    Method::from_str(&name).map_err(|e| syn::Error::new(attr.span(), e))?,
                ));

                let parser = get_parser(&mut builder);

                attr.parse_args_with(parser)?
            }
        }
        let properties = builder.build().map_err(|e| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to build FreshRouteAttributes: {}", e),
            )
        })?;
        Ok(Some(FreshRouteAttributes {
            method: properties.method,
            path: properties.path,
            headers: properties.headers,
            timeout: properties.timeout,
            connect_timeout: properties.connect_timeout,
            read_timeout: properties.read_timeout,
        }))
    }
}

impl crate::parser::Parser<ItemTrait> for MethodMetaParser {
    type Output = Vec<MethodMeta>;

    fn parse(trait_item: &ItemTrait) -> syn::Result<Self::Output> {
        let mut out = Vec::new();
        for item in &trait_item.items {
            let TraitItem::Fn(method) = item else {
                continue;
            };

            let sig_ident = method.sig.ident.clone();

            let ok_ty = extract_ok_type(&method.sig.output).unwrap_or_else(|| {
                panic!("Method {} must have return type Result<T, E>", sig_ident)
            });

            // 解析路由属性
            let route = FreshRouteAttributeParser::parse(&method.attrs)?;

            // 如果没有路由属性，跳过该方法
            let Some(mut route) = route else {
                continue;
            };

            route.set_path_if_none(format!("/{}", &sig_ident.to_string()));

            // 解析参数属性
            let params = ParamMetaParser::parse(&method.sig.inputs)?;

            out.push(
                MethodMeta {
                    sig_ident,
                    ok_ty,
                    params,
                    route,
                }
            )
        }
        Ok(out)
    }
}

impl crate::parser::Parser<Punctuated<FnArg, syn::token::Comma>> for ParamMetaParser {
    type Output = Vec<ParamMeta>;

    fn parse(inputs: &Punctuated<FnArg, Comma>) -> syn::Result<Self::Output> {
        let mut params = Vec::new();

        for input in inputs {
            if let FnArg::Typed(pt) = input {
                let ident = match &*pt.pat {
                    Pat::Ident(pi) => pi.ident.clone(),
                    _ => panic!("Unsupported parameter pattern"),
                };
                let mut kind = ParamKind::Other;
                let mut header_name: Option<String> = None;
                for a in &pt.attrs {
                    let name = a.path().get_ident().map(|i| i.to_string()).unwrap_or_default();
                    match name.as_str() {
                        "path" => kind = ParamKind::Path,
                        "query" => kind = ParamKind::Query,
                        "json" => kind = ParamKind::Json,
                        "header" => {
                            if let Some(syn::Lit::Str(s)) = a.parse_args().ok() {
                                header_name = Some(s.value());
                            }
                        }
                        _ => {}
                    }
                }
                if let Some(h) = header_name {
                    kind = ParamKind::Header(h);
                }
                params.push(ParamMeta { ident, ty: Some((*pt.ty).clone()), kind });
            }
        }

        Ok(params)
    }
}

fn get_parser<'a>(builder: &'a mut AttributePropertiesBuilder) -> impl Parser<Output = ()> + 'a {
    syn::meta::parser(move |meta| {
        let path = meta.path.clone();
        let ident_str = path.get_ident().map(|i| i.to_string().to_lowercase());

        match ident_str.as_deref() {
            Some("endpoint") => {
                let lit: LitStr = meta.value()?.parse()?;
                builder.endpoint(Some(lit.value()));
            }
            Some("path") => {
                let lit: LitStr = meta.value()?.parse()?;
                builder.path(Some(lit.value()));
            }
            Some("headers") => {
                let mut headers = Vec::new();
                meta.parse_nested_meta(|nested| {
                    let key = nested
                        .path
                        .get_ident()
                        .ok_or_else(|| nested.error("Expected identifier"))?
                        .to_string();
                    let val: LitStr = nested.value()?.parse()?;
                    headers.push((key, val.value()));
                    Ok(())
                })?;
                builder.headers(headers);
            }
            Some("timeout") => {
                let lit: LitInt = meta.value()?.parse()?;
                builder.timeout(Some(lit.base10_parse()?));
            }
            Some("connect_timeout") => {
                let lit: LitInt = meta.value()?.parse()?;
                builder.connect_timeout(Some(lit.base10_parse()?));
            }
            Some("read_timeout") => {
                let lit: LitInt = meta.value()?.parse()?;
                builder.read_timeout(Some(lit.base10_parse()?));
            }
            _ => {}
        }
        Ok(())
    })
}
