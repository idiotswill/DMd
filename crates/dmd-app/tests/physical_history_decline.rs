#[path = "support/historical_physical/mod.rs"]
mod historical;

#[tokio::test]
async fn original_v4_decline_history_and_genuine_v5_continuation() {
    Box::pin(historical::run("inspiration-decline-v4")).await;
}
