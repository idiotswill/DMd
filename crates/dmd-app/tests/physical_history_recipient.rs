#[path = "support/historical_physical/mod.rs"]
mod historical;

#[tokio::test]
async fn original_v4_recipient_history_and_genuine_v5_continuation() {
    Box::pin(historical::run("inspiration-recipient-v4")).await;
}
