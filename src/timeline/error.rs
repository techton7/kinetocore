//! Error types for timeline parsing, keyframe validation, and interval compilation.

use std::time::Duration;

/// Errors that can occur during timeline authoring, position parsing, keyframe validation, and interval compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineError {
    /// Two spanned clips on the same track overlap in time.
    TrackIntervalOverlap {
        /// Name of the conflicting track.
        track: String,
        /// Start time of the first spanned clip.
        start1: Duration,
        /// End time of the first spanned clip.
        end1: Duration,
        /// Start time of the second spanned clip.
        start2: Duration,
        /// End time of the second spanned clip.
        end2: Duration,
    },
    /// A position referenced a label that has not been defined.
    UnknownLabel(String),
    /// A position string failed to parse or was mathematically invalid.
    InvalidPosition(String),
    /// A keyframe sequence failed validation (non-monotonic, invalid boundary, etc.).
    InvalidKeyframe(String),
    /// A requested track was not found in the timeline.
    TrackNotFound(String),
    /// The timeline is empty (contains no tracks or clips).
    EmptyTimeline,
}

impl std::fmt::Display for TimelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TrackIntervalOverlap {
                track,
                start1,
                end1,
                start2,
                end2,
            } => {
                write!(
                    f,
                    "Track interval overlap on track '{track}': [{start1:?}, {end1:?}) overlaps with [{start2:?}, {end2:?})"
                )
            }
            Self::UnknownLabel(label) => {
                write!(f, "Unknown label '{label}' in timeline")
            }
            Self::InvalidPosition(msg) => {
                write!(f, "Invalid position: {msg}")
            }
            Self::InvalidKeyframe(msg) => {
                write!(f, "Invalid keyframe: {msg}")
            }
            Self::TrackNotFound(track) => {
                write!(f, "Track '{track}' not found in timeline")
            }
            Self::EmptyTimeline => {
                write!(f, "Timeline is empty (contains no tracks or clips)")
            }
        }
    }
}

impl std::error::Error for TimelineError {}
