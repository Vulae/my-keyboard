/// Sleeps until an instant, returning the duration we slept.
/// If the instant is in the past it will not sleep and return a duration of zero.
pub fn sleep_until(instant: std::time::Instant) -> std::time::Duration {
    let now = std::time::Instant::now();
    let dur = instant.duration_since(now);
    if dur == std::time::Duration::ZERO {
        return std::time::Duration::ZERO;
    }
    std::thread::sleep(dur);
    dur
}
