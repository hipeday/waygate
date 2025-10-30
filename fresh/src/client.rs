use reqwest::Client;
use std::time::Duration;
use url::Url;

#[derive(Clone, Debug)]
pub struct HttpClientOption {
    endpoint: Url,                  // 端点 URL
    timeout: Duration,              // 可选的请求超时
    headers: Vec<(String, String)>, // 额外基础请求头
    read_timeout: Duration,         // 读取超时
    connect_timeout: Duration,      // 连接超时
}

impl HttpClientOption {
    /// 获取端点 URL
    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }

    /// 获取请求超时
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// 获取额外请求头
    pub fn headers(&self) -> &Vec<(String, String)> {
        &self.headers
    }

    /// 获取读取超时
    pub fn read_timeout(&self) -> Duration {
        self.read_timeout
    }

    /// 获取连接超时
    pub fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }
}

impl HttpClientOption {

    pub fn with_endpoint(endpoint: Url) -> HttpClientOption {
        HttpClientOption {
            endpoint,
            timeout: default_timeout(),
            headers: default_headers(),
            read_timeout: default_read_timeout(),
            connect_timeout: default_connect_timeout(),
        }
    }
}

impl Default for HttpClientOption {
    fn default() -> Self {
        HttpClientOption {
            endpoint: Url::parse("http://localhost").expect("Valid default endpoint"),
            timeout: default_timeout(),
            headers: default_headers(),
            read_timeout: default_read_timeout(),
            connect_timeout: default_connect_timeout(),
        }
    }
}

/// HttpClientOption 的构建器
#[derive(Clone, Debug, Default)]
pub struct HttpClientOptionBuilder {
    // 使用 String 持有 endpoint，在 build() 时统一解析为 Url
    endpoint: Option<String>,
    timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    headers: Vec<(String, String)>,
}

impl HttpClientOption {
    /// 创建一个新的 Builder
    pub fn builder() -> HttpClientOptionBuilder {
        HttpClientOptionBuilder::default()
    }

    /// 基于现有配置生成 Builder（便于在此基础上修改）
    pub fn to_builder(&self) -> HttpClientOptionBuilder {
        HttpClientOptionBuilder {
            endpoint: Some(self.endpoint.to_string()),
            timeout: Some(self.timeout),
            read_timeout: Some(self.read_timeout),
            connect_timeout: Some(self.connect_timeout),
            headers: self.headers.clone(),
        }
    }
}

impl HttpClientOptionBuilder {
    /// 直接设置 endpoint 字符串（推荐）
    pub fn endpoint(mut self, endpoint: impl AsRef<str>) -> Self {
        self.endpoint = Some(endpoint.as_ref().to_string());
        self
    }

    /// 以 Url 形式设置 endpoint
    pub fn endpoint_url(mut self, endpoint: Url) -> Self {
        self.endpoint = Some(endpoint.to_string());
        self
    }

    /// 设置总请求超时（整体 deadline）
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 设置读取超时（空闲/无数据可读）
    pub fn read_timeout(mut self, read_timeout: Duration) -> Self {
        self.read_timeout = Some(read_timeout);
        self
    }

    /// 设置连接超时（建连/握手）
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = Some(connect_timeout);
        self
    }

    /// 追加一个基础请求头
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// 一次性追加多组请求头
    pub fn headers<I, K, V>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.headers
            .extend(iter.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// 构建最终配置
    ///
    /// - 若未显式设置 endpoint，则使用 HttpClientOption::default() 的默认值。
    /// - 若 endpoint 设置为无效 URL，将返回错误。
    pub fn build(self) -> crate::error::Result<HttpClientOption> {

        let mut opt = HttpClientOption::default();

        let Some(ep) = self.endpoint else {
            return Err(crate::error::Error::InvalidArgument("Endpoint cannot be empty".to_string()));
        };

        opt.endpoint = Url::parse(&ep)?;

        if let Some(t) = self.timeout {
            opt.timeout = t;
        }
        if let Some(rt) = self.read_timeout {
            opt.read_timeout = rt;
        }
        if let Some(ct) = self.connect_timeout {
            opt.connect_timeout = ct;
        }

        if !self.headers.is_empty() {
            // 若要覆盖而不是追加，可改为直接赋值：opt.headers = self.headers;
            opt.headers.extend(self.headers);
        }

        Ok(opt)
    }
}

fn default_timeout() -> Duration {
    Duration::from_secs(6)
}

fn default_headers() -> Vec<(String, String)> {
    // User-Agent: fresh-client/{version}
    vec![(
        String::from("User-Agent"),
        format!("fresh-client/{}", env!("CARGO_PKG_VERSION")),
    )]
}

fn default_read_timeout() -> Duration {
    Duration::from_secs(6)
}

fn default_connect_timeout() -> Duration {
    Duration::from_secs(6)
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
                reqwest::header::HeaderValue::from_str(&value)?,
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
