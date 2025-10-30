use proc_macro::TokenStream;

mod param;
mod method;
mod util;
mod expand;
mod parser;

/// trait 宏入口：`#[fresh(...)]`
/// 只在入口使用 `proc_macro::TokenStream`，内部统一用 `proc_macro2::TokenStream`
#[proc_macro_attribute]
pub fn fresh(attr: TokenStream, item: TokenStream) -> TokenStream {
    let out: proc_macro2::TokenStream = expand::expand_fresh(attr.into(), item.into());
    out.into()
}