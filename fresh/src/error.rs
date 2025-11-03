use thiserror::Error;

/// 定义错误类型
#[derive(Debug, Error)]
pub enum Error {
    // 传输层/超时/DNS 等，直接透传 reqwest::Error 的错误
    #[error(transparent)]
    Transport(#[from] reqwest::Error),

    // 非 2xx 状态码 附带 URL、状态码和响应体片段
    #[error("HTTP error: {status} for URL: {url}\nResponse body (truncated): {body_snippet}")]
    Http {
        url: String,
        status: reqwest::StatusCode,
        body_snippet: String,
    },

    // JSON(或其它) 解析错误，附带 URL、源错误以及响应体片段
    #[error("Failed to parse response from URL: {url}\nSource error: {source}\nResponse body (truncated): {body_snippet}")]
    Decode {
        url: String,
        #[source]
        source: serde_json::Error,
        body_snippet: String,
    },

    // URL 解析错误，附带源错误
    #[error("Invalid URL: {0}")]
    UrlParse(#[from] url::ParseError),

    // 请求头错误，附带源错误
    #[error("Invalid header: {0}")]
    HeaderName(#[from] reqwest::header::InvalidHeaderName),

    // 请求头值错误，附带源错误
    #[error("Invalid header value: {0}")]
    HeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    // 非法参数错误，附带描述信息
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    // Format 错误，附带描述信息
    #[error("Format error: {0}")]
    FormatError(String),

}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// 截取响应体的前 N 个字符用于错误消息 避免过长导致日志臃肿
pub fn snippet(s: &str, limit: usize) -> String {
    const ELLIPSIS: &str = "…";
    if s.len() <= limit {
        s.to_string()
    } else {
        let mut out = s.chars().take(limit).collect::<String>();
        out.push_str(ELLIPSIS);
        out
    }
}