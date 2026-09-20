//! Core value-level tween implementation.

use std::time::Duration;

use crate::clock::{AnimClock, ClockState};
use crate::direction::PlaybackDirection;
use crate::ease::Ease;
use crate::interpolate::Interpolate;
use crate::repeat::{RepeatCount, RepeatStrategy};

/// A pure-value tween animating between two points of type `T`.
#[derive(Debug, Clone, PartialEq)]
pub struct Tween<T: Interpolate> {
    from: T,
    to: T,
    ease: Ease,
    clock: AnimClock,
}

impl<T: Interpolate> Tween<T> {
    /// Create a zero-duration tween setting a target value immediately.
    ///
    /// This corresponds to GSAP's `set()` semantics. The tween is immediately
    /// completed at creation and returns `value` upon evaluation.
    #[inline]
    pub fn set(value: T) -> Self {
        Self {
            from: value.clone(),
            to: value,
            ease: Ease::Linear,
            clock: AnimClock::new(Duration::ZERO),
        }
    }

    /// Create a deterministic tween animating from `from` to `to` over `duration`.
    ///
    /// This corresponds to GSAP's `fromTo()` semantics.
    #[inline]
    pub fn from_to(from: T, to: T, duration: Duration) -> Self {
        Self {
            from,
            to,
            ease: Ease::default(),
            clock: AnimClock::new(duration),
        }
    }

    /// Set easing curve.
    #[inline]
    pub fn ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }

    /// Set repeat count.
    #[inline]
    pub fn repeat(mut self, repeat_count: impl Into<RepeatCount>) -> Self {
        self.clock = self.clock.with_repeat_count(repeat_count);
        self
    }

    /// Set repeat strategy.
    #[inline]
    pub fn repeat_strategy(mut self, repeat_strategy: RepeatStrategy) -> Self {
        self.clock = self.clock.with_repeat_strategy(repeat_strategy);
        self
    }

    /// Enable or disable yoyo (mirrored repeat) playback.
    #[inline]
    pub fn yoyo(mut self, yoyo: bool) -> Self {
        if yoyo {
            self.clock = self.clock.with_repeat_strategy(RepeatStrategy::MirroredRepeat);
        } else {
            self.clock = self.clock.with_repeat_strategy(RepeatStrategy::Repeat);
        }
        self
    }

    /// Set playback direction.
    #[inline]
    pub fn direction(mut self, direction: PlaybackDirection) -> Self {
        self.clock = self.clock.with_direction(direction);
        self
    }

    /// Get reference to start value.
    #[inline]
    pub fn from_value(&self) -> &T {
        &self.from
    }

    /// Get reference to end value.
    #[inline]
    pub fn to_value(&self) -> &T {
        &self.to
    }

    /// Get easing curve.
    #[inline]
    pub fn easing(&self) -> Ease {
        self.ease
    }

    /// Get reference to internal animation clock.
    #[inline]
    pub fn clock(&self) -> &AnimClock {
        &self.clock
    }

    /// Get mutable reference to internal animation clock.
    #[inline]
    pub fn clock_mut(&mut self) -> &mut AnimClock {
        &mut self.clock
    }

    /// Check if the tween has completed its playback.
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.clock.is_completed()
    }

    /// Current linear progress fraction in `[0.0, 1.0]`.
    #[inline]
    pub fn progress(&self) -> f32 {
        self.clock.cycle_fraction()
    }

    /// Current eased progress fraction in `[0.0, 1.0]`.
    #[inline]
    pub fn eased_progress(&self) -> f32 {
        self.ease.sample(self.clock.mirrored_cycle_fraction())
    }

    /// Calculate the current value at the current clock position.
    #[inline]
    pub fn value(&self) -> T {
        let eased_ratio = self.eased_progress();
        self.from.interpolate(&self.to, eased_ratio)
    }

    /// Sample the tween at an arbitrary elapsed duration without mutating internal state.
    pub fn sample_at(&self, time: Duration) -> T {
        let mut temp_clock = self.clock.clone();
        temp_clock.seek(time);
        let eased_ratio = self.ease.sample(temp_clock.mirrored_cycle_fraction());
        self.from.interpolate(&self.to, eased_ratio)
    }

    /// Advance the tween by `delta` time and return the newly interpolated value.
    pub fn step(&mut self, delta: Duration) -> (T, ClockState) {
        let state = self.clock.tick(delta);
        (self.value(), state)
    }

    /// Seek the tween to a specific elapsed duration and return the newly interpolated value.
    pub fn seek(&mut self, time: Duration) -> T {
        self.clock.seek(time);
        self.value()
    }

    /// Reverse playback direction.
    #[inline]
    pub fn reverse(&mut self) {
        self.clock.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tween_set_immediate() {
        let mut tween = Tween::set(42.0f32);
        assert!(tween.is_completed());
        assert_eq!(tween.value(), 42.0);

        let (val, state) = tween.step(Duration::from_secs(1));
        assert_eq!(val, 42.0);
        assert_eq!(state, ClockState::Completed);
    }

    #[test]
    fn test_tween_from_to_linear() {
        let mut tween = Tween::from_to(0.0f32, 100.0f32, Duration::from_secs(4))
            .ease(Ease::Linear);

        assert_eq!(tween.value(), 0.0);
        assert!(!tween.is_completed());

        // 25% (1 sec)
        let (v25, state) = tween.step(Duration::from_secs(1));
        assert_eq!(state, ClockState::Active);
        assert_eq!(v25, 25.0);

        // 50% (2 sec)
        let (v50, state) = tween.step(Duration::from_secs(1));
        assert_eq!(state, ClockState::Active);
        assert_eq!(v50, 50.0);

        // 75% (3 sec)
        let (v75, state) = tween.step(Duration::from_secs(1));
        assert_eq!(state, ClockState::Active);
        assert_eq!(v75, 75.0);

        // 100% (4 sec)
        let (v100, state) = tween.step(Duration::from_secs(1));
        assert_eq!(state, ClockState::Completed);
        assert_eq!(v100, 100.0);
        assert!(tween.is_completed());
    }

    #[test]
    fn test_tween_sample_at() {
        let tween = Tween::from_to(10.0f64, 50.0f64, Duration::from_secs(2))
            .ease(Ease::Linear);

        assert_eq!(tween.sample_at(Duration::ZERO), 10.0);
        assert_eq!(tween.sample_at(Duration::from_secs(1)), 30.0);
        assert_eq!(tween.sample_at(Duration::from_secs(2)), 50.0);

        // State remains unchanged
        assert_eq!(tween.value(), 10.0);
    }

    #[test]
    fn test_tween_yoyo() {
        let mut tween = Tween::from_to(0.0f32, 10.0f32, Duration::from_secs(1))
            .ease(Ease::Linear)
            .repeat(2)
            .yoyo(true);

        // Forward: 0s -> 0.0
        assert_eq!(tween.value(), 0.0);

        // 0.5s -> 5.0
        let (v, _) = tween.step(Duration::from_millis(500));
        assert_eq!(v, 5.0);

        // 1.0s -> 10.0
        let (v, _) = tween.step(Duration::from_millis(500));
        assert_eq!(v, 10.0);

        // 1.5s -> 5.0 (yoyo back)
        let (v, _) = tween.step(Duration::from_millis(500));
        assert_eq!(v, 5.0);

        // 2.0s -> 0.0 (finished)
        let (v, state) = tween.step(Duration::from_millis(500));
        assert_eq!(v, 0.0);
        assert_eq!(state, ClockState::Completed);
        assert!(tween.is_completed());
    }

    #[test]
    fn test_tween_seek_and_reverse() {
        let mut tween = Tween::from_to([0.0f32, 0.0f32], [100.0f32, 200.0f32], Duration::from_secs(4))
            .ease(Ease::Linear);

        // Seek to 2 sec (50%)
        let v = tween.seek(Duration::from_secs(2));
        assert_eq!(v, [50.0, 100.0]);

        // Reverse and step 1 sec back
        tween.reverse();
        let (v, state) = tween.step(Duration::from_secs(1));
        assert_eq!(state, ClockState::Active);
        assert_eq!(v, [25.0, 50.0]);
    }
}
