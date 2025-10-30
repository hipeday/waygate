use fresh_macros::fresh;

#[allow(async_fn_in_trait)]
#[fresh(endpoint = "https://httpbin.org", headers(foo = "bar", user_agent = "fresh-test"))]
pub trait FreshAttribute {
    #[get("/get")]
    async fn search(&self, #[query] q: crate::SearchQuery) -> fresh::Result<crate::HttpBinGet>;
}