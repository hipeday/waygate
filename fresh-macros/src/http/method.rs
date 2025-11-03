use std::str::FromStr;
use quote::quote;

/// HTTP 请求方法枚举
#[derive(Debug, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Head,
    Options,
    Delete,
    Patch,
    Trace,
}

impl Method {
    pub fn to_token(&self) -> proc_macro2::TokenStream {
        match self {
            Method::Get => quote! { ::fresh::reqwest::Method::GET },
            Method::Post => quote! { ::fresh::reqwest::Method::POST },
            Method::Put => quote! { ::fresh::reqwest::Method::PUT },
            Method::Head => quote! { ::fresh::reqwest::Method::HEAD },
            Method::Options => quote! { ::fresh::reqwest::Method::OPTIONS },
            Method::Delete => quote! { ::fresh::reqwest::Method::DELETE },
            Method::Patch => quote! { ::fresh::reqwest::Method::PATCH },
            Method::Trace => quote! { ::fresh::reqwest::Method::TRACE },
        }
    }
}

impl FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            "TRACE" => Ok(Method::Trace),
            _ => Err(format!("Unsupported HTTP method: {}", s)),
        }
    }
}

impl From<String> for Method {
    fn from(value: String) -> Self {
        Method::from_str(&value).unwrap_or(Method::Get)
    }
}