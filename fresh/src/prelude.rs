pub use crate::{
    error::{Error, Result},
    client::{HttpClient, HttpClientOption, HttpClientOptionBuilder}
};

// 若有可选特性，可按需导出
// #[cfg(feature = "blocking")]
// pub use crate::client::blocking::BlockingHttpClient;

// 未来可放：常用 trait（如拦截器、解码策略等）
// pub use crate::codec::DecodeExt;