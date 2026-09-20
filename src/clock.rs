//! Time tracking and cycle arithmetic engine for animations.

use std::time::Duration;

use crate::direction::PlaybackDirection;
use crate::repeat::{RepeatCount, RepeatStrategy};

/// Playback state of an [`AnimClock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockState {
    /// The clock is actively ticking and has not reached completion.
    Active,
    /// The clock reached its termination condition (finite duration elapsed or reversed to 0).
    Completed,
}

/// Headless timekeeper managing playback progress, repeating cycles, and direction.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimClock {
    elapsed: Duration,
    cycle_duration: Duration,
    repeat_count: RepeatCount,
    repeat_strategy: RepeatStrategy,
    direction: PlaybackDirection,
}

impl AnimClock {
    /// Create a new clock for an animation with the given cycle duration.
    #[inline]
    pub fn new(cycle_duration: Duration) -> Self {
        Self {
            elapsed: Duration::ZERO,
            cycle_duration,
            repeat_count: RepeatCount::default(),
            repeat_strategy: RepeatStrategy::default(),
            direction: PlaybackDirection::default(),
        }
    }

    /// Set repeat count configuration.
    #[inline]
    pub fn with_repeat_count(mut self, repeat_count: impl Into<RepeatCount>) -> Self {
        self.repeat_count = repeat_count.into();
        self
    }

    /// Set repeat strategy (standard or mirrored ping-pong).
    #[inline]
    pub fn with_repeat_strategy(mut self, repeat_strategy: RepeatStrategy) -> Self {
        self.repeat_strategy = repeat_strategy;
        self
    }

    /// Set playback direction.
    #[inline]
    pub fn with_direction(mut self, direction: PlaybackDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Get the duration of a single cycle.
    #[inline]
    pub fn cycle_duration(&self) -> Duration {
        self.cycle_duration
    }

    /// Get current total elapsed time.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Get current playback direction.
    #[inline]
    pub fn direction(&self) -> PlaybackDirection {
        self.direction
    }

    /// Set current playback direction.
    #[inline]
    pub fn set_direction(&mut self, direction: PlaybackDirection) {
        self.direction = direction;
    }

    /// Reverse current playback direction.
    #[inline]
    pub fn reverse(&mut self) {
        self.direction.toggle();
    }

    /// Get total playback duration if finite, or `None` if infinite.
    #[inline]
    pub fn total_duration(&self) -> Option<Duration> {
        self.repeat_count.total_duration(self.cycle_duration)
    }

    /// Returns `true` if the clock has completed its configured duration.
    #[inline]
    pub fn is_completed(&self) -> bool {
        if self.cycle_duration.is_zero() {
            return true;
        }

        match self.total_duration() {
            Some(total) => {
                if self.direction.is_forward() {
                    self.elapsed >= total
                } else {
                    self.elapsed.is_zero()
                }
            }
            None => false,
        }
    }

    /// Seek to an arbitrary elapsed duration.
    #[inline]
    pub fn seek(&mut self, elapsed: Duration) {
        if let Some(total) = self.total_duration() {
            self.elapsed = elapsed.min(total);
        } else {
            self.elapsed = elapsed;
        }
    }

    /// Advance the internal clock by `delta`.
    ///
    /// If playing backward, `delta` is subtracted from elapsed time.
    pub fn tick(&mut self, delta: Duration) -> ClockState {
        if self.cycle_duration.is_zero() {
            return ClockState::Completed;
        }

        if self.direction.is_forward() {
            self.elapsed = self.elapsed.saturating_add(delta);
            if let Some(total) = self.total_duration()
                && self.elapsed >= total
            {
                self.elapsed = total;
                return ClockState::Completed;
            }
        } else {
            self.elapsed = self.elapsed.saturating_sub(delta);
            if self.elapsed.is_zero() {
                return ClockState::Completed;
            }
        }

        ClockState::Active
    }

    /// Returns the normalized linear progress fraction within the current cycle in `[0.0, 1.0]`.
    pub fn cycle_fraction(&self) -> f32 {
        if self.cycle_duration.is_zero() {
            return 1.0;
        }

        let cycle_nanos = self.cycle_duration.as_nanos() as f64;
        let elapsed_nanos = self.elapsed.as_nanos() as f64;

        if let Some(total) = self.total_duration()
            && self.elapsed >= total
        {
            return 1.0;
        }

        let remainder = elapsed_nanos % cycle_nanos;
        let frac = (remainder / cycle_nanos) as f32;
        frac.clamp(0.0, 1.0)
    }

    /// Returns the current cycle fraction taking into account [`RepeatStrategy::MirroredRepeat`].
    ///
    /// If the strategy is [`RepeatStrategy::MirroredRepeat`] and the animation is currently
    /// in an odd cycle index (1, 3, 5...), the fraction is mirrored: `1.0 - fraction`.
    pub fn mirrored_cycle_fraction(&self) -> f32 {
        let linear_frac = self.cycle_fraction();

        if self.repeat_strategy != RepeatStrategy::MirroredRepeat || self.cycle_duration.is_zero() {
            return linear_frac;
        }

        let cycle_nanos = self.cycle_duration.as_nanos() as f64;
        let elapsed_nanos = self.elapsed.as_nanos() as f64;

        let cycle_index = if let Some(total) = self.total_duration() {
            if self.elapsed >= total {
                let total_nanos = total.as_nanos() as f64;
                let count = (total_nanos / cycle_nanos).floor() as u64;
                if count > 0 && total_nanos % cycle_nanos == 0.0 {
                    count - 1
                } else {
                    count
                }
            } else {
                (elapsed_nanos / cycle_nanos).floor() as u64
            }
        } else {
            (elapsed_nanos / cycle_nanos).floor() as u64
        };

        if cycle_index % 2 == 1 {
            1.0 - linear_frac
        } else {
            linear_frac
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_duration_clock() {
        let mut clock = AnimClock::new(Duration::ZERO);
        assert!(clock.is_completed());
        assert_eq!(clock.cycle_fraction(), 1.0);
        assert_eq!(clock.mirrored_cycle_fraction(), 1.0);
        assert_eq!(clock.tick(Duration::from_secs(1)), ClockState::Completed);
    }

    #[test]
    fn test_forward_ticking_and_completion() {
        let mut clock = AnimClock::new(Duration::from_secs(2));
        assert!(!clock.is_completed());
        assert_eq!(clock.cycle_fraction(), 0.0);

        assert_eq!(clock.tick(Duration::from_secs(1)), ClockState::Active);
        assert_eq!(clock.elapsed(), Duration::from_secs(1));
        assert!((clock.cycle_fraction() - 0.5).abs() < 1e-4);

        assert_eq!(clock.tick(Duration::from_secs(1)), ClockState::Completed);
        assert_eq!(clock.elapsed(), Duration::from_secs(2));
        assert!(clock.is_completed());
        assert_eq!(clock.cycle_fraction(), 1.0);
    }

    #[test]
    fn test_mirrored_yoyo_repeat() {
        let mut clock = AnimClock::new(Duration::from_secs(1))
            .with_repeat_count(2)
            .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        assert_eq!(clock.total_duration(), Some(Duration::from_secs(2)));

        // At 0.0s
        assert_eq!(clock.mirrored_cycle_fraction(), 0.0);

        // At 0.5s -> 0.5
        clock.tick(Duration::from_millis(500));
        assert!((clock.mirrored_cycle_fraction() - 0.5).abs() < 1e-4);

        // At 1.0s -> 1.0
        clock.tick(Duration::from_millis(500));
        assert!((clock.mirrored_cycle_fraction() - 1.0).abs() < 1e-4);

        // At 1.5s -> cycle 2 (odd index 1), fraction is 0.5 -> mirrored is 0.5
        clock.tick(Duration::from_millis(500));
        assert!((clock.mirrored_cycle_fraction() - 0.5).abs() < 1e-4);

        // At 2.0s -> completed, mirrored is 0.0
        clock.tick(Duration::from_millis(500));
        assert!(clock.is_completed());
        assert!((clock.mirrored_cycle_fraction() - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_seek_and_reverse() {
        let mut clock = AnimClock::new(Duration::from_secs(4));

        clock.seek(Duration::from_secs(3));
        assert_eq!(clock.elapsed(), Duration::from_secs(3));
        assert!((clock.cycle_fraction() - 0.75).abs() < 1e-4);

        clock.reverse();
        assert!(clock.direction().is_backward());

        clock.tick(Duration::from_secs(1));
        assert_eq!(clock.elapsed(), Duration::from_secs(2));
        assert!((clock.cycle_fraction() - 0.5).abs() < 1e-4);

        clock.tick(Duration::from_secs(2));
        assert_eq!(clock.elapsed(), Duration::ZERO);
        assert!(clock.is_completed());
    }
}
