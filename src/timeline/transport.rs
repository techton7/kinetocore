//! Master clock transport coordinating playback state, seeking, and repeat cycles.

use std::time::Duration;

use crate::clock::{AnimClock, ClockState};
use crate::direction::PlaybackDirection;
use crate::repeat::{RepeatCount, RepeatStrategy};

/// Playback state of a [`TimelineTransport`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    /// Playhead is actively advancing.
    Playing,
    /// Playhead is paused.
    Paused,
    /// Playhead reached its completion condition.
    Settled,
}

/// Headless master transport playhead managing timeline playback, seeking, and repeats.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineTransport {
    clock: AnimClock,
    is_playing: bool,
    time_scale: f64,
}

impl TimelineTransport {
    /// Create a new transport playhead for a timeline with `duration`.
    ///
    /// By default, the transport starts in the playing state with normal playback speed (`1.0`).
    pub fn new(duration: Duration) -> Self {
        Self {
            clock: AnimClock::new(duration),
            is_playing: true,
            time_scale: 1.0,
        }
    }

    /// Create a new transport playhead starting in the paused state.
    pub fn new_paused(duration: Duration) -> Self {
        Self {
            clock: AnimClock::new(duration),
            is_playing: false,
            time_scale: 1.0,
        }
    }

    /// Start or resume playback.
    ///
    /// If the transport previously completed at the end in forward mode,
    /// calling `play()` automatically restarts playback from `0`.
    pub fn play(&mut self) {
        if self.clock.is_completed() {
            if self.clock.direction().is_forward() {
                self.clock.seek(Duration::ZERO);
            } else if let Some(total) = self.clock.total_duration() {
                self.clock.seek(total);
            }
        }
        self.is_playing = true;
    }

    /// Pause playback.
    #[inline]
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// Toggle between playing and paused states.
    pub fn toggle(&mut self) {
        if self.is_playing {
            self.pause();
        } else {
            self.play();
        }
    }

    /// Reverse playback direction.
    #[inline]
    pub fn reverse(&mut self) {
        self.clock.reverse();
    }

    /// Restart playback from the beginning (`t = 0`) in forward direction.
    pub fn restart(&mut self) {
        self.clock.set_direction(PlaybackDirection::Forward);
        self.clock.seek(Duration::ZERO);
        self.is_playing = true;
    }

    /// Seek directly to an absolute timestamp `time`.
    #[inline]
    pub fn seek(&mut self, time: Duration) {
        self.clock.seek(time);
    }

    /// Set normalized progress fraction in `[0.0, 1.0]`.
    pub fn set_progress(&mut self, progress: f32) {
        let p = progress.clamp(0.0, 1.0);
        let cycle = self.clock.cycle_duration();
        if cycle.is_zero() {
            self.clock.seek(Duration::ZERO);
        } else {
            let nanos = (cycle.as_nanos() as f64 * p as f64).round() as u64;
            self.clock.seek(Duration::from_nanos(nanos));
        }
    }

    /// Set playback speed multiplier (`1.0` is normal speed).
    #[inline]
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0);
    }

    /// Get current playback speed multiplier.
    #[inline]
    pub fn time_scale(&self) -> f64 {
        self.time_scale
    }

    /// Set repeat count and strategy (standard or mirrored yoyo).
    pub fn set_repeat(&mut self, count: impl Into<RepeatCount>, strategy: RepeatStrategy) {
        self.clock = self
            .clock
            .clone()
            .with_repeat_count(count)
            .with_repeat_strategy(strategy);
    }

    /// Current effective cycle sampling timestamp within `[0, duration]`.
    ///
    /// Accounts for cycle wrapping and yoyo reversal with nanosecond numerical precision.
    pub fn time(&self) -> Duration {
        let cycle = self.clock.cycle_duration();
        if cycle.is_zero() {
            return Duration::ZERO;
        }

        let elapsed = self.clock.elapsed();
        if let Some(total) = self.clock.total_duration()
            && elapsed >= total
        {
            if self.clock.repeat_strategy() == RepeatStrategy::MirroredRepeat {
                let cycle_nanos = cycle.as_nanos();
                let total_nanos = total.as_nanos();
                let count = (total_nanos / cycle_nanos) as u64;
                let cycle_index = if count > 0 && total_nanos % cycle_nanos == 0 {
                    count - 1
                } else {
                    count
                };
                if cycle_index % 2 == 1 {
                    return Duration::ZERO;
                }
            }
            return cycle;
        }

        let cycle_nanos = cycle.as_nanos();
        let elapsed_nanos = elapsed.as_nanos();
        let cycle_index = (elapsed_nanos / cycle_nanos) as u64;
        let remainder_nanos = (elapsed_nanos % cycle_nanos) as u64;

        if self.clock.repeat_strategy() == RepeatStrategy::MirroredRepeat && cycle_index % 2 == 1 {
            let mirrored_nanos = (cycle_nanos as u64).saturating_sub(remainder_nanos);
            Duration::from_nanos(mirrored_nanos)
        } else {
            Duration::from_nanos(remainder_nanos)
        }
    }

    /// Total elapsed playback time across all repeat cycles.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    /// Total playback duration if finite, or `None` if repeating infinitely.
    #[inline]
    pub fn total_duration(&self) -> Option<Duration> {
        self.clock.total_duration()
    }

    /// Access reference to the underlying [`AnimClock`].
    #[inline]
    pub fn clock(&self) -> &AnimClock {
        &self.clock
    }

    /// Current normalized progress fraction in `[0.0, 1.0]`.
    #[inline]
    pub fn progress(&self) -> f32 {
        self.clock.mirrored_cycle_fraction()
    }

    /// Returns `true` if the transport is actively playing.
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.is_playing && !self.clock.is_completed()
    }

    /// Returns `true` if playback is paused.
    #[inline]
    pub fn is_paused(&self) -> bool {
        !self.is_playing
    }

    /// Returns `true` if playback has reached its termination condition.
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.clock.is_completed()
    }

    /// Get the current high-level transport state.
    #[inline]
    pub fn state(&self) -> TransportState {
        if self.is_completed() {
            TransportState::Settled
        } else if self.is_playing {
            TransportState::Playing
        } else {
            TransportState::Paused
        }
    }

    /// Get current playback direction.
    #[inline]
    pub fn direction(&self) -> PlaybackDirection {
        self.clock.direction()
    }

    /// Set playback direction.
    #[inline]
    pub fn set_direction(&mut self, direction: PlaybackDirection) {
        self.clock.set_direction(direction);
    }

    /// Advance the playhead by `dt`.
    ///
    /// Multiplies `dt` by `time_scale`, advances internal timekeeping,
    /// and handles completion state transitions.
    pub fn step(&mut self, dt: Duration) -> ClockState {
        if !self.is_playing {
            return if self.is_completed() {
                ClockState::Completed
            } else {
                ClockState::Active
            };
        }

        let scaled_nanos = (dt.as_nanos() as f64 * self.time_scale).round() as u64;
        let scaled_dt = Duration::from_nanos(scaled_nanos);

        let state = self.clock.tick(scaled_dt);
        if state == ClockState::Completed {
            self.is_playing = false;
        }
        state
    }
}
