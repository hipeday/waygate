use url::Url;
use reqwest::Client;
use std::time::Duration;
use derive_builder::Builder;

#[derive(Clone, Debug, Builder)]
pub struct HttpClientOption {
    #[builder(setter(custom))]
    pub endpoint: Url,                  // 端点 URL
    #[builder(default = "Duration::from_secs(6)")]
    pub timeout: Duration,              // 可选的请求超时
    #[builder(default = "default_headers()")]
    pub headers: Vec<(String, String)>, // 额外基础请求头
    #[builder(default = "Duration::from_secs(6)")]
    pub read_timeout: Duration,         // 读取超时
    #[builder(default = "Duration::from_secs(6)")]
    pub connect_timeout: Duration,      // 连接超时
}

impl HttpClientOption {
    pub fn builder() -> HttpClientOptionBuilder {
        HttpClientOptionBuilder::default()
    }
}

impl HttpClientOptionBuilder {
    fn endpoint_setter(&mut self, endpoint: impl AsRef<str>) -> &mut Self {
        let endpoint = Url::parse(endpoint.as_ref()).expect(format!("Invalid endpoint '{}'", endpoint.as_ref()).as_str());
        self.endpoint = Some(endpoint);
        self
    }

    pub fn endpoint(&mut self, endpoint: impl AsRef<str>) -> &mut Self {
        let endpoint = Url::parse(endpoint.as_ref()).expect(format!("Invalid endpoint '{}'", endpoint.as_ref()).as_str());
        self.endpoint = Some(endpoint);
        self
    }
}

impl HttpClientOption {
    pub fn with_endpoint(endpoint: impl AsRef<str>) -> HttpClientOption {
        HttpClientOption::builder()
            .endpoint_setter(endpoint)
            .build()
            .unwrap()
    }
}

fn default_headers() -> Vec<(String, String)> {
    // User-Agent: fresh-client/{version}
    vec![(
        String::from("user-agent"),
        format!("fresh-client/{}", env!("CARGO_PKG_VERSION")),
    )]
}

fn build_client(
    headers: reqwest::header::HeaderMap,
    timeout: Duration,
    connect_timeout: Duration,
    read_timeout: Duration,
) -> crate::error::Result<Client> {
    let client = Client::builder()
        .default_headers(headers)
        .timeout(timeout)
        .connect_timeout(connect_timeout)
        .read_timeout(read_timeout)
        .build()?;

    Ok(client)
}

/// HTTP 客户端封装，基于 reqwest 实现
pub struct HttpClient {
    inner: Client,
    option: HttpClientOption,
}

impl HttpClient {
    /// 创建一个新的 HttpClient 实例
    pub fn new(option: HttpClientOption) -> crate::error::Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();

        for (header, value) in &option.headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(header.as_bytes())?,
                // 兼容非 ASCII 的值（如中文）：优先 from_str，失败则回退到原始字节
                reqwest::header::HeaderValue::from_str(&value)
                    .or_else(|_| reqwest::header::HeaderValue::from_bytes(value.as_bytes()))?,
            );
        }

        let inner = build_client(headers, option.timeout, option.connect_timeout, option.read_timeout)?;

        Ok(Self {
            inner,
            option,
        })
    }

    pub fn with_endpoint(endpoint: impl AsRef<str>) -> crate::error::Result<Self> {
        let endpoint = Url::parse(endpoint.as_ref())?;
        let option = HttpClientOption::with_endpoint(endpoint);
        Self::new(option)
    }

    pub fn from_reqwest(inner: Client, endpoint: impl AsRef<str>) -> crate::error::Result<Self> {
        let endpoint = Url::parse(endpoint.as_ref())?;
        Ok(Self {
            inner,
            option: HttpClientOption::with_endpoint(endpoint),
        })
    }

    pub fn client(&self) -> &Client {
        &self.inner
    }

    pub fn endpoint(&self) -> &Url {
        &self.option.endpoint
    }

    pub fn options(&self) -> &HttpClientOption {
        &self.option
    }
}