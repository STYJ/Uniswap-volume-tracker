
mod environment;
mod telegram;

#[tokio::main]
async fn main() {
    environment::load();
    telegram::run().await;
}