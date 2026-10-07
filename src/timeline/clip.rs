//! Timeline clip representations separating instant updates from spanned animations.

use std::time::Duration;

use crate::ease::Ease;
use crate::interpolate::Interpolate;
use crate::timeline::keyframe::Keyframe;

/// Spanned clips with finite, non-zero durations.
#[derive(Debug, Clone, PartialEq)]
pub enum SpannedClipKind<T: Interpolate> {
    /// Continuous tween between `from` and `to` using an easing curve.
    Tween {
        /// Starting value at `start_time`.
        from: T,
        /// Ending value at `start_time + duration`.
        to: T,
        /// Easing formula applied during interpolation.
        ease: Ease,
    },
    /// Constant value held across the clip's duration.
    Hold {
        /// Constant value to maintain.
        value: T,
    },
    /// Multi-keyframe sequence with per-segment easing curves.
    Keyframes {
        /// Monotonically sorted keyframes spanning normalized `[0.0, 1.0]`.
        keyframes: Vec<Keyframe<T>>,
    },
}

/// Instant clips taking place at a discrete point in time (`duration == 0`).
#[derive(Debug, Clone, PartialEq)]
pub enum InstantClipKind<T: Interpolate> {
    /// Discrete set operation immediately applying `value` at `start_time`.
    Set {
        /// Target value to assign.
        value: T,
    },
}

/// Kind of timeline clip: either Instant or Spanned.
#[derive(Debug, Clone, PartialEq)]
pub enum ClipKind<T: Interpolate> {
    /// Zero-duration instant clip.
    Instant(InstantClipKind<T>),
    /// Finite-duration spanned clip.
    Spanned(SpannedClipKind<T>),
}

/// A resolved clip positioned along a track's timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Clip<T: Interpolate> {
    /// Monotonic unique identifier for this clip.
    pub id: usize,
    /// Absolute timestamp when this clip begins.
    pub start_time: Duration,
    /// Duration of the clip (`Duration::ZERO` for Instant clips).
    pub duration: Duration,
    /// Specific clip variant and parameters.
    pub kind: ClipKind<T>,
}

impl<T: Interpolate> Clip<T> {
    /// Create an instant `Set` clip.
    #[inline]
    pub fn set(id: usize, start_time: Duration, value: T) -> Self {
        Self {
            id,
            start_time,
            duration: Duration::ZERO,
            kind: ClipKind::Instant(InstantClipKind::Set { value }),
        }
    }

    /// Create a spanned `Tween` clip.
    #[inline]
    pub fn tween(
        id: usize,
        start_time: Duration,
        duration: Duration,
        from: T,
        to: T,
        ease: Ease,
    ) -> Self {
        Self {
            id,
            start_time,
            duration,
            kind: ClipKind::Spanned(SpannedClipKind::Tween { from, to, ease }),
        }
    }

    /// Create a spanned `Hold` clip.
    #[inline]
    pub fn hold(id: usize, start_time: Duration, duration: Duration, value: T) -> Self {
        Self {
            id,
            start_time,
            duration,
            kind: ClipKind::Spanned(SpannedClipKind::Hold { value }),
        }
    }

    /// Create a spanned `Keyframes` clip.
    #[inline]
    pub fn keyframes(
        id: usize,
        start_time: Duration,
        duration: Duration,
        keyframes: Vec<Keyframe<T>>,
    ) -> Self {
        Self {
            id,
            start_time,
            duration,
            kind: ClipKind::Spanned(SpannedClipKind::Keyframes { keyframes }),
        }
    }

    /// Calculate the end time of this clip (`start_time + duration`).
    #[inline]
    pub fn end_time(&self) -> Duration {
        self.start_time + self.duration
    }

    /// Returns `true` if this clip is an instant clip (`duration == 0`).
    #[inline]
    pub fn is_instant(&self) -> bool {
        matches!(self.kind, ClipKind::Instant(_))
    }

    /// Returns `true` if this clip is a spanned clip (`duration > 0`).
    #[inline]
    pub fn is_spanned(&self) -> bool {
        matches!(self.kind, ClipKind::Spanned(_))
    }

    /// Return the starting value of this clip.
    pub fn start_value(&self) -> T {
        match &self.kind {
            ClipKind::Instant(InstantClipKind::Set { value }) => value.clone(),
            ClipKind::Spanned(SpannedClipKind::Tween { from, .. }) => from.clone(),
            ClipKind::Spanned(SpannedClipKind::Hold { value }) => value.clone(),
            ClipKind::Spanned(SpannedClipKind::Keyframes { keyframes }) => {
                keyframes
                    .first()
                    .expect("keyframe sequence must contain at least one keyframe")
                    .value
                    .clone()
            }
        }
    }

    /// Return the terminal (ending) value of this clip.
    pub fn terminal_value(&self) -> T {
        match &self.kind {
            ClipKind::Instant(InstantClipKind::Set { value }) => value.clone(),
            ClipKind::Spanned(SpannedClipKind::Tween { to, .. }) => to.clone(),
            ClipKind::Spanned(SpannedClipKind::Hold { value }) => value.clone(),
            ClipKind::Spanned(SpannedClipKind::Keyframes { keyframes }) => {
                keyframes
                    .last()
                    .expect("keyframe sequence must contain at least one keyframe")
                    .value
                    .clone()
            }
        }
    }

    /// Sample this clip at `time` within its active span.
    pub fn sample_at(&self, time: Duration) -> T {
        match &self.kind {
            ClipKind::Instant(InstantClipKind::Set { value }) => value.clone(),
            ClipKind::Spanned(SpannedClipKind::Hold { value }) => value.clone(),
            ClipKind::Spanned(SpannedClipKind::Tween { from, to, ease }) => {
                if self.duration.is_zero() || time <= self.start_time {
                    return from.clone();
                }
                if time >= self.end_time() {
                    return to.clone();
                }
                let elapsed_nanos = (time - self.start_time).as_nanos() as f64;
                let dur_nanos = self.duration.as_nanos() as f64;
                let tau = (elapsed_nanos / dur_nanos).clamp(0.0, 1.0) as f32;
                let ratio = ease.sample(tau);
                from.interpolate(to, ratio)
            }
            ClipKind::Spanned(SpannedClipKind::Keyframes { keyframes }) => {
                if keyframes.is_empty() {
                    panic!("keyframe sequence must not be empty");
                }
                if self.duration.is_zero() || time <= self.start_time {
                    return keyframes[0].value.clone();
                }
                if time >= self.end_time() {
                    return keyframes[keyframes.len() - 1].value.clone();
                }
                let elapsed_nanos = (time - self.start_time).as_nanos() as f64;
                let dur_nanos = self.duration.as_nanos() as f64;
                let tau = (elapsed_nanos / dur_nanos).clamp(0.0, 1.0) as f32;

                for i in 0..keyframes.len() - 1 {
                    let k_curr = &keyframes[i];
                    let k_next = &keyframes[i + 1];
                    if tau <= k_next.offset {
                        let span = k_next.offset - k_curr.offset;
                        let seg_tau = if span <= 0.0 {
                            0.0
                        } else {
                            ((tau - k_curr.offset) / span).clamp(0.0, 1.0)
                        };
                        let ratio = k_curr.ease.sample(seg_tau);
                        return k_curr.value.interpolate(&k_next.value, ratio);
                    }
                }

                keyframes[keyframes.len() - 1].value.clone()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_constructors_and_helpers() {
        let set_clip = Clip::set(0, Duration::from_secs(1), 42.0f64);
        assert!(set_clip.is_instant());
        assert!(!set_clip.is_spanned());
        assert_eq!(set_clip.duration, Duration::ZERO);
        assert_eq!(set_clip.end_time(), Duration::from_secs(1));

        let tween_clip = Clip::tween(
            1,
            Duration::from_secs(2),
            Duration::from_secs(3),
            0.0f32,
            1.0f32,
            Ease::Linear,
        );
        assert!(!tween_clip.is_instant());
        assert!(tween_clip.is_spanned());
        assert_eq!(tween_clip.end_time(), Duration::from_secs(5));
    }
}
