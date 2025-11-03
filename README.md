# Fresh - 声明式 HTTP 客户端

[![Crates.io](https://img.shields.io/crates/v/fresh.svg)](https://crates.io/crates/fresh)
[![Docs.rs](https://docs.rs/fresh/badge.svg)](https://docs.rs/fresh)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

基于 reqwest 的`Retrofit 风格`声明式 HTTP 客户端。用 trait + 注解描述接口，过程宏生成具体调用代码。

- 简单：用 `#[fresh(...)]` 标注 trait，一键生成 `XxxClient`
- 直观：方法上用 `#[get]` / `#[post]` 指定 HTTP 动作与路径
- 安全：编译期展开，零运行时反射
- 轻量：基于 reqwest，无侵入

## 安装与特性

工作区内已默认将 `fresh-macros` 作为可选依赖并通过特性启用。对外 crate 使用方式：

```toml
[dependencies]
fresh = { version = "0.1.0", features = ["macros"] } # macros 缺省已开启；可按需关闭/开启
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

若要禁用并按需开启：

```toml
[dependencies]
fresh = { version = "0.1.0", default-features = false, features = ["macros"] }
```

在本仓库中，`fresh` 已对宏进行根导出，可使用 `fresh::request`。

## 快速开始

```rust
use serde::{Deserialize, Serialize};
use fresh::fresh;

#[derive(Debug, Serialize)]
struct SearchQuery {
    q: String,
    page: u32,
}

#[derive(Debug, Deserialize)]
struct HttpBinGet {
    url: String,
    args: serde_json::Value,
    headers: serde_json::Value,
}

#[request(
    endpoint = "https://httpbin.org",
    // headers 的键名支持下划线写法，宏会规范为短横线小写（user_agent -> user-agent）
    headers(foo = "bar", user_agent = "fresh-test"),
    timeout = 10000,
    connect_timeout = 11000,
    read_timeout = 12000,
)]
trait Api {
    #[get(
        path = "/get",
        // headers 的键名支持下划线写法，宏会规范为短横线小写（user_agent -> user-agent）
        headers(user_agent = "fresh-client/0.1", x_token = "demo-token"), // 方法级别 headers 会覆盖 trait 级别同名请求头对应的值
        timeout = 10000, // 函数级别超时时间会覆盖 trait 级别
        connect_timeout = 11000, // 函数级别超时时间会覆盖 trait 级别
        read_timeout = 12000,  // 函数级别超时时间会覆盖 trait 级别
    )]
    async fn search(&self, #[query] q: SearchQuery) -> fresh::Result<HttpBinGet>;
}
```

宏将生成 `ApiClient`，并注入构造方法：

- `ApiClient::with_endpoint("&str")`
- `ApiClient::new_default()` 使用 trait 上的 `endpoint` 与 `headers` 构造

## 运行示例与测试

运行示例：

```bash
cargo run --example hello_world
```

运行测试（含宏测试与实际访问 httpbin 的用例）：

```bash
cargo test -p fresh-test
```

## 运行时 API（摘）

`HttpClientOption` 提供 Builder 构造：

```rust
let opt = fresh::HttpClientOption::builder()
    .endpoint("https://httpbin.org")
    .header("user-agent", "fresh-client/0.1")
    .headers(vec![("x-token", "demo-token")])
    .build()?; // endpoint 不能为空
let client = fresh::HttpClient::new(opt)?;
```

注意：
- `build()` 要求 `endpoint` 必填；若希望提供默认端点，可在你自己的调用侧封装。
- 默认会附加 `User-Agent: fresh-client/{version}`。

## 关于请求头的大小写与字符集

- 键名：宏会将 `headers(user_agent = "...")` 等键名规范为短横线小写（`user-agent`）。
- 值：HTTP 协议规范推荐 ASCII。库在内部优先用 `HeaderValue::from_str`，若失败会回退用原始字节构造以兼容中文，但对端可能按 ISO-8859-1/ASCII 展示导致“乱码”。建议仅在必要时于头部放非 ASCII，或考虑将信息放入 body/query。

## 设计约束与建议

- 公开 trait 中使用 `async fn` 会触发编译器建议（`async_fn_in_trait`）。你可以：
  - 在 trait 上加 `#[allow(async_fn_in_trait)]`（仓库中测试已如此处理）
  - 或改为返回 `impl Future<Output = ...> + Send` 的签名（更稳健）
- `fresh` 根已导出：`HttpClient`、`HttpClientOption`、`HttpClientOptionBuilder`、`fresh::fresh`（受特性 `macros` 控制）。

## 许可证

本项目使用 [MIT License](LICENSE)。