#[tokio::main]
pub async fn main() {
    gateway::run().await;
}
