//! End-to-end test of the public `speedy_core` pipeline as the Kotlin shell
//! drives it: repeated counter samples -> smoothed rate -> formatted icon+label.

use speedy_core::App;

const SEC: i64 = 1_000_000_000;

#[test]
fn realistic_download_session() {
    let mut app = App::new(64, 64);
    app.tick(0, 0, 0); // seed

    // Simulate ~2 MB/s download, ~256 KB/s upload for 10 seconds.
    let (mut rx, mut tx) = (0i64, 0i64);
    for i in 1..=10 {
        rx += 2 * 1024 * 1024;
        tx += 256 * 1024;
        let icon = app.tick(rx, tx, i * SEC);
        assert_eq!(icon.len(), 64 * 64);
    }

    let label = app.label();
    // After convergence the label should read megabytes down, kilobytes up.
    assert!(label.contains('M'), "label was: {label}");
    assert!(label.contains('K'), "label was: {label}");
    assert!(label.starts_with('↓'));
}

#[test]
fn going_idle_returns_to_zero() {
    let mut app = App::new(48, 48);
    app.tick(0, 0, 0);
    app.tick(5 * 1024 * 1024, 0, SEC); // a burst

    // No traffic for a while -> EMA decays back into the sub-1-byte deadzone.
    let rx = 5 * 1024 * 1024;
    for i in 2..=60 {
        let icon = app.tick(rx, 0, i * SEC); // rx unchanged => zero new bytes
        assert_eq!(icon.len(), 48 * 48);
    }
    assert!(app.label().contains("↓ 0/s"), "label was: {}", app.label());
}
