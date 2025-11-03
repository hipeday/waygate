//! 代码生成主流程（syn v2 兼容版）。
//!
//! 流程：
//! 1) 解析 `#[fresh(...)]` 的 trait 级参数（目前支持 base_url）
//! 2) 收集方法/参数注解元信息
//! 3) 剥离自定义注解，输出“干净的 trait”
//! 4) 生成 `XxxClient` 结构与构造函数（with_base_url/new_default）
//! 5) 为每个方法展开基于 reqwest 的实际调用代码

mod fresh;

/// 宏输入类型枚举
pub enum MacroForm {
    /// #[proc_macro_attribute]
    ///
    /// #[fresh(endpoint = "...", ...)]
    Attribute {
        attr: proc_macro2::TokenStream,
        item: proc_macro2::TokenStream,
    },
    /// #[proc_macro_derive]
    /// #[derive(...)]
    _Derive {
        item: proc_macro2::TokenStream,
    },
    /// #[proc_macro]
    /// 函数式宏
    _Function {
        item: proc_macro2::TokenStream,
    },
}

/// 具体使用的宏枚举
pub enum MacroKind {
    Fresh,
}

/// 宏调用信息结构体
pub struct MacroCall {
    pub kind: MacroKind,
    pub form: MacroForm,
}

impl MacroCall {
    pub fn new(kind: MacroKind, form: MacroForm) -> Self {
        Self { kind, form }
    }
}

/// 展开器 trait
pub trait Expander {
    fn expand(&self, call: MacroCall) -> syn::Result<proc_macro2::TokenStream>;
}

/// dispatch 宏展开
pub fn dispatch(call: MacroCall) -> proc_macro2::TokenStream {
    let expander: Box<dyn Expander> = match call.kind {
        MacroKind::Fresh => Box::new(fresh::FreshExpander {}),
    };
    expander.expand(call).unwrap_or_else(|e| e.to_compile_error())
}
