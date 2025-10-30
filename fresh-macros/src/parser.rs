// 解析器

mod fresh_attribute;

pub use fresh_attribute::{FreshAttributeParser};

/// 解析器 trait
pub trait Parser<I> {
    type Output;

    fn parse(input: &I) -> syn::Result<Self::Output>;
}