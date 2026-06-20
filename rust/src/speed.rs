//! Network throughput computation.
//!
//! Turns the cumulative byte counters reported by `android.net.TrafficStats`
//! into smoothed per-second download/upload rates. This module is pure: it has
//! no Android dependency and is fully host-testable.

/// Exponential-moving-average smoothing factor in `0.0..=1.0`.
/// Higher reacts faster to bursts; lower is steadier. 0.4 feels responsive
/// without flickering on a ~1 Hz update.
const EMA_ALPHA: f64 = 0.4;

#[inline]
fn ema(prev: f64, sample: f64) -> f64 {
    EMA_ALPHA * sample + (1.0 - EMA_ALPHA) * prev
}

/// Smoothed throughput estimator driven by periodic counter samples.
#[derive(Debug, Clone, Default)]
pub struct SpeedState {
    initialized: bool,
    prev_rx: u64,
    prev_tx: u64,
    prev_ns: u64,
    /// Last smoothed download rate in bytes/sec.
    pub down_bps: f64,
    /// Last smoothed upload rate in bytes/sec.
    pub up_bps: f64,
}

impl SpeedState {
    pub fn new() -> Self {
        Self::default()
    }

    fn seed(&mut self, rx: u64, tx: u64, ns: u64) {
        self.prev_rx = rx;
        self.prev_tx = tx;
        self.prev_ns = ns;
        self.initialized = true;
    }

    /// Feed the latest cumulative counters (bytes) and a monotonic timestamp
    /// (nanoseconds, e.g. `SystemClock.elapsedRealtimeNanos()`).
    ///
    /// `rx`/`tx` are `i64` to mirror the Java API: a negative value means
    /// `TrafficStats.UNSUPPORTED` and is treated as "no new data" — the last
    /// smoothed rates are returned unchanged.
    ///
    /// Returns the smoothed `(download, upload)` rates in bytes/sec.
    pub fn tick(&mut self, rx: i64, tx: i64, ns: i64) -> (f64, f64) {
        // Unsupported counters: hold the previous estimate rather than spiking.
        if rx < 0 || tx < 0 || ns < 0 {
            return (self.down_bps, self.up_bps);
        }
        let (rx, tx, ns) = (rx as u64, tx as u64, ns as u64);

        if !self.initialized {
            self.seed(rx, tx, ns);
            return (0.0, 0.0);
        }

        let dt = ns.saturating_sub(self.prev_ns) as f64 / 1e9;
        if dt <= 0.0 {
            // Clock didn't advance (duplicate sample) — nothing to recompute.
            return (self.down_bps, self.up_bps);
        }

        match (rx.checked_sub(self.prev_rx), tx.checked_sub(self.prev_tx)) {
            (Some(d_rx), Some(d_tx)) => {
                self.down_bps = ema(self.down_bps, d_rx as f64 / dt);
                self.up_bps = ema(self.up_bps, d_tx as f64 / dt);
            }
            // A counter went backwards => device/counter reset (e.g. reboot).
            // Reseed from this sample and report zero for this tick.
            _ => {
                self.down_bps = 0.0;
                self.up_bps = 0.0;
            }
        }

        self.prev_rx = rx;
        self.prev_tx = tx;
        self.prev_ns = ns;
        (self.down_bps, self.up_bps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEC: i64 = 1_000_000_000;

    #[test]
    fn first_tick_seeds_and_reports_zero() {
        let mut s = SpeedState::new();
        assert_eq!(s.tick(1000, 500, 0), (0.0, 0.0));
    }

    #[test]
    fn steady_rate_converges_to_truth() {
        let mut s = SpeedState::new();
        s.tick(0, 0, 0);
        // 1 MB/s down, 100 KB/s up, every second.
        let (mut rx, mut tx) = (0i64, 0i64);
        let mut down = 0.0;
        let mut up = 0.0;
        for i in 1..=30 {
            rx += 1_000_000;
            tx += 100_000;
            let r = s.tick(rx, tx, i * SEC);
            down = r.0;
            up = r.1;
        }
        // EMA should be essentially exact after 30 identical samples.
        assert!((down - 1_000_000.0).abs() < 1.0, "down={down}");
        assert!((up - 100_000.0).abs() < 1.0, "up={up}");
    }

    #[test]
    fn zero_elapsed_holds_previous() {
        let mut s = SpeedState::new();
        s.tick(0, 0, 0);
        s.tick(1_000_000, 0, SEC); // establish a non-zero rate
        let before = s.down_bps;
        let (down, _) = s.tick(2_000_000, 0, SEC); // same timestamp
        assert_eq!(down, before);
    }

    #[test]
    fn counter_reset_yields_zero_not_garbage() {
        let mut s = SpeedState::new();
        s.tick(5_000_000, 5_000_000, 0);
        s.tick(6_000_000, 6_000_000, SEC);
        // Counters drop below previous (reboot) -> must not produce a huge spike.
        let (down, up) = s.tick(10, 10, 2 * SEC);
        assert_eq!((down, up), (0.0, 0.0));
    }

    #[test]
    fn unsupported_counter_holds_previous() {
        let mut s = SpeedState::new();
        s.tick(0, 0, 0);
        s.tick(1_000_000, 0, SEC);
        let before = (s.down_bps, s.up_bps);
        assert_eq!(s.tick(-1, -1, 2 * SEC), before);
    }
}
