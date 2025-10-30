use serde::{Deserialize, Serialize};
use fresh::fresh;

#[derive(Debug, Serialize)]
struct SearchQuery {
    q: String,
    page: u32,
}

#[derive(Debug, Serialize)]
struct CreateUser {
    name: String,
}

#[derive(Debug, Deserialize)]
struct HttpBinGet {
    url: String,
    args: serde_json::Value,
    headers: serde_json::Value,
}

#[fresh(endpoint = "https://httpbin.org", headers(user_agent = "fresh-client/0.1hhh", x_token = "这是token"))]
trait Api {
    #[get("/get")]
    async fn search(&self, #[query] q: SearchQuery) -> fresh::Result<HttpBinGet>;

    #[post("/post")]
    async fn create_user(&self, #[json] body: CreateUser) -> fresh::Result<serde_json::Value>;

    #[get("/anything/{id}")]
    async fn anything(
        &self,
        #[path] id: u64,
        #[header("X-Trace-Id")] trace: String,
    ) -> fresh::Result<serde_json::Value>;
}

#[tokio::main]
async fn main() -> fresh::Result<()> {
    // 方式一：使用 trait 上的 base_url（宏会生成 new_default）
    let api = ApiClient::new_default()?;

    // 方式二：显式传入 base_url
    // let api = ApiClient::with_base_url("https://httpbin.org")?;

    let out = api
        .search(SearchQuery { q: "rust".into(), page: 1 })
        .await?;
    println!("GET /get => url={}, args={}, header={}", out.url, out.args, out.headers);

    let created = api
        .create_user(CreateUser { name: "Ferris".into() })
        .await?;
    println!("POST /post => {}", created);

    let any = api.anything(42, "trace-123".into()).await?;
    println!("GET /anything/42 => {}", any);

    Ok(())
}