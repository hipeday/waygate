use fresh_test::{
    macros::{FreshAttribute, FreshAttributeClient},
    SearchQuery,
};

#[test]
fn test_fresh_attribute_parsing() {
    let client = FreshAttributeClient::new_default().unwrap();
    let endpoint = client.core.endpoint();
    let headers = client.core.options().headers();
    for (key, value) in headers {
        match key.as_str() {
            "foo" => assert_eq!(value, "bar"),
            "user-agent" => assert_eq!(value, "fresh-test"),
            "User-Agent" => {} // 默认头，允许存在
            _ => panic!("Unexpected header: {}: {}", key, value),
        }
    }
    assert_eq!(endpoint.as_str(), "https://httpbin.org/");
}

#[tokio::test]
async fn test_search() {
    let client = FreshAttributeClient::new_default().unwrap();
    let response = client.search(SearchQuery { q: "test".into(), page: 1 }).await.unwrap();
    assert_eq!(response.url, "https://httpbin.org/get?q=test&page=1");
    let args = response.args;
    assert_eq!(args["q"], "test");
    assert_eq!(args["page"], "1");
}