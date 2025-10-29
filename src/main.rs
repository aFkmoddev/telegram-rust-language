pub mod interpreter;
pub mod builtins;
pub mod bot;

#[tokio::main]
async fn main() {
    bot::telegram_bot().await;
}
