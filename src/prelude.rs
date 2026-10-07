//! Convenience re-exports of core kinetocore types.

pub use crate::clock::{AnimClock, ClockState};
pub use crate::direction::PlaybackDirection;
pub use crate::ease::Ease;
pub use crate::interpolate::{lerp, Interpolate};
pub use crate::repeat::{RepeatCount, RepeatStrategy};
pub use crate::spring::{DampingRegime, Spring, SpringConfig, SpringError, SpringState};
pub use crate::state::TweenEndpoints;
pub use crate::target::{IntoTargetSampler, Target, TargetSampler};
pub use crate::timeline::{
    validate_keyframes, Clip, ClipKind, CompiledTimeline, CompiledTrack, InstantClipKind,
    Keyframe, OffsetSign, Position, SignedDuration, SpannedClipKind, TimelineBuilder,
    TimelineError, TrackBuilder,
};
pub use crate::tween::Tween;
