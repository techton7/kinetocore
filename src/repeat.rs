//! Repeat counts and repeat strategy definitions.

use std::time::Duration;

/// Specifies how many times an animation cycle repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatCount {
    /// Repeat a finite number of cycles.
    ///
    /// `Finite(1)` means play once without repeating (the default).
    /// `Finite(2)` means play the cycle twice.
    Finite(u32),

    /// Repeat until a specific total duration has elapsed.
    ///
    /// If the duration is not an exact multiple of the cycle duration,
    /// the final cycle may be truncated.
    For(Duration),

    /// Repeat indefinitely.
    Infinite,
}

impl Default for RepeatCount {
    #[inline]
    fn default() -> Self {
        Self::Finite(1)
    }
}

impl From<u32> for RepeatCount {
    #[inline]
    fn from(count: u32) -> Self {
        Self::Finite(count)
    }
}

impl From<Duration> for RepeatCount {
    #[inline]
    fn from(duration: Duration) -> Self {
        Self::For(duration)
    }
}

impl RepeatCount {
    /// Calculate total duration if finite, or `None` if infinite.
    #[inline]
    pub fn total_duration(&self, cycle_duration: Duration) -> Option<Duration> {
        match self {
            RepeatCount::Finite(count) => Some(cycle_duration * *count),
            RepeatCount::For(dur) => Some(*dur),
            RepeatCount::Infinite => None,
        }
    }
}

/// Strategy for repeating animation cycles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatStrategy {
    /// Restart from the beginning at each cycle (0.0 -> 1.0, 0.0 -> 1.0).
    #[default]
    Repeat,

    /// Ping-pong / yoyo between endpoints (0.0 -> 1.0, 1.0 -> 0.0).
    MirroredRepeat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat_count_total_duration() {
        let cycle = Duration::from_secs(2);

        assert_eq!(
            RepeatCount::Finite(3).total_duration(cycle),
            Some(Duration::from_secs(6))
        );
        assert_eq!(
            RepeatCount::For(Duration::from_secs(5)).total_duration(cycle),
            Some(Duration::from_secs(5))
        );
        assert_eq!(RepeatCount::Infinite.total_duration(cycle), None);
    }

    #[test]
    fn test_repeat_conversions() {
        assert_eq!(RepeatCount::from(5u32), RepeatCount::Finite(5));
        assert_eq!(
            RepeatCount::from(Duration::from_millis(500)),
            RepeatCount::For(Duration::from_millis(500))
        );
    }
}
