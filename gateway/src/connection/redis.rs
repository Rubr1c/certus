use bb8_redis::RedisConnectionManager;

pub async fn create_pool(url: &String) -> bb8::Pool<RedisConnectionManager> {
    let manager = RedisConnectionManager::new(url.as_str())
        .expect("Failed to open redis client");

    bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to make connection bb8 pool")
}
