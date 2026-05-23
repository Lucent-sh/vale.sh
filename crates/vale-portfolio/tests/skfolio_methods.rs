//! Integration tests for skfolio subprocess methods (skipped without Python).

#[tokio::test]
#[ignore = "requires python3 + skfolio"]
async fn skfolio_hrp_returns_weights() {
    let tickers = vec!["A".into(), "B".into(), "C".into()];
    let returns = serde_json::json!([
        [0.01, 0.02, 0.01],
        [-0.01, 0.01, -0.02],
        [0.02, -0.01, 0.03],
        [0.00, 0.02, 0.01],
    ]);
    let w = vale_portfolio::skfolio::optimize_via_skfolio(
        "hrp",
        &returns.to_string(),
        &tickers,
    )
    .await
    .expect("skfolio");
    assert_eq!(w.len(), 3);
    let sum: f64 = w.iter().map(|(_, v)| v).sum();
    assert!((sum - 1.0).abs() < 0.05);
}
