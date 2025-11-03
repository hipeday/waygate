use std::time::Duration;
use fresh_test::{
    macros::{FreshAttribute, FreshAttributeClient},
    SearchQuery,
};

#[test]
fn test_fresh_attribute_parsing() {
    let client = FreshAttributeClient::new_default().unwrap();
    let endpoint = client.core.endpoint();
    let options = client.core.options();
    let headers = &options.headers;
    for (key, value) in headers {
        match key.as_str() {
            "foo" => assert_eq!(value, "bar"),
            "user-agent" => assert_eq!(value, "fresh-test"),
            "User-Agent" => {} // 默认头，允许存在
            _ => panic!("Unexpected header: {}: {}", key, value),
        }
    }
    println!("options.connect_timeout(): {:?}", options.connect_timeout);
    assert_eq!(endpoint.as_str(), "https://httpbin.org/");
    assert_eq!(options.timeout, Duration::from_millis(10000));
    assert_eq!(options.connect_timeout, Duration::from_millis(11000)); // 默认值
    assert_eq!(options.read_timeout, Duration::from_millis(12000)); // 默认值
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