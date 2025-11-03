//! 宏内部的通用工具。
//!
//! - 提取 `Result<T, E>` 的 `T`
//! - 将字符串 HTTP 方法名映射为 `reqwest::Method` 代码片段

use syn::{ReturnType, Type};

/// 从返回类型中提取 `Result<T, E>` 的 `T`。
pub fn extract_ok_type(ret: &ReturnType) -> Option<proc_macro2::TokenStream> {
    let ReturnType::Type(_, ty) = ret else { return None; };
    if let Type::Path(tp) = &**ty {
        let seg = tp.path.segments.last()?;
        if seg.ident == "Result" {
            if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                if let Some(syn::GenericArgument::Type(t)) = ab.args.first() {
                    return Some(quote::quote! { #t });
                }
            }
        }
    }
    None
}