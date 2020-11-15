
mod environment;
mod telegram;

#[tokio::main]
async fn main() {
    environment::load();
    telegram::start_bot().await;
}