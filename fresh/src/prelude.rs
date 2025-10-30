//! fresh 的常用导出集合（建议用户 `use fresh::prelude::*;`）
//! 只导出高频、稳定的对外 API，避免污染命名空间。

pub use crate::{
    error::{Error, Result},
    client::{HttpClient, HttpClientOption, HttpClientOptionBuilder}
};

// 若有可选特性，可按需导出
// #[cfg(feature = "blocking")]
// pub use crate::client::blocking::BlockingHttpClient;

// 未来可放：常用 trait（如拦截器、解码策略等）
// pub use crate::codec::DecodeExt;