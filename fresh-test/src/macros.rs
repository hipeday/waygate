use fresh_macros::request;

#[allow(async_fn_in_trait)]
#[request(
    endpoint = "https://httpbin.org",
    headers(foo = "bar", user_agent = "fresh-test"),
    timeout = 10000,
    connect_timeout = 11000,
    read_timeout = 12000,
)]
pub trait FreshAttribute {

    #[get(
        path = "/get",
        headers(foo = "bar", user_agent = "fresh-test"),
        timeout = 13000,
    )]
    async fn search(&self, #[query] q: crate::SearchQuery) -> fresh::Result<crate::HttpBinGet>;
}