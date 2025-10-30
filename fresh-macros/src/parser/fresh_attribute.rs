use proc_macro2::TokenStream;
use syn::{
    parse::Parser as _,
    spanned::Spanned,
    LitStr
};

// 接口级宏上的属性解析器
pub struct FreshAttributeParser;

/// 接口级解析属性
#[derive(Debug)]
pub struct FreshAttributes {
    pub endpoint: Option<String>,       // 基础端点 URL
    pub headers: Vec<(String, String)>, // 额外请求头
}

impl crate::parser::Parser<TokenStream> for FreshAttributeParser {
    type Output = FreshAttributes;

    fn parse(input: &TokenStream) -> syn::Result<Self::Output> {
        let mut endpoint: Option<String> = None; // 基础端点 URL
        let mut headers: Vec<(String, String)> = Vec::new(); // 额外请求头

        let parser = syn::meta::parser(|meta| {
            let path = meta.path.clone();
            match path.get_ident() {
                None => {
                    return Err(syn::Error::new("Expected identifier".span(), "Expected identifier"));
                }
                Some(ident) => {
                    match ident.to_string().as_str() {
                        "endpoint" => {
                            // 读取等号右侧的字面量字符串
                            let lit: LitStr = meta.value()?.parse()?;
                            endpoint = Some(lit.value());
                        }
                        "headers" => {
                            // 解析 headers(foo = "bar", baz = "qux")
                            meta.parse_nested_meta(|nested| -> syn::Result<()> {
                                // 左侧必须是标识符（属性语法限制），不能是字符串字面量
                                let Some(inner_ident) = nested.path.get_ident() else {
                                    return Err(nested.error("headers(...) 的键必须是标识符，例如 headers(user_agent = \"...\")"));
                                };
                                let key = inner_ident.to_string();
                                let val: LitStr = nested.value()?.parse()?;
                                headers.push((key, val.value()));
                                Ok(())
                            })?;
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
            Ok(())
        });

        parser.parse2(input.clone())?;
        Ok(FreshAttributes { endpoint, headers })
    }
}
