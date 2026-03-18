use bb8_redis::RedisConnectionManager;

#[inline(always)]
pub async fn create_pool(url: &str) -> bb8::Pool<RedisConnectionManager> {
    let manager =
        RedisConnectionManager::new(url).expect("Failed to open redis client");

    bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to make connection bb8 pool")
}
