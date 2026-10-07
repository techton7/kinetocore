//! Compiled timeline representations with flat, validated, allocation-free intervals.

use std::collections::HashMap;
use std::time::Duration;

use crate::interpolate::Interpolate;
use crate::timeline::clip::Clip;
use crate::timeline::error::TimelineError;

/// Type-erased trait for querying compiled tracks without knowing their target generic type.
pub trait AnyCompiledTrack: std::any::Any + Send + Sync {
    /// Return the name of the track.
    fn name(&self) -> &str;
    /// Return the total duration of the track.
    fn duration(&self) -> Duration;
    /// Return the total number of clips on the track.
    fn clip_count(&self) -> usize;
    /// Returns `true` if the clip at `idx` is spanned.
    fn is_spanned_at(&self, idx: usize) -> bool;
    /// Return the `(start_time, end_time)` interval of the clip at `idx`.
    fn clip_interval(&self, idx: usize) -> Option<(Duration, Duration)>;
    /// Upcast this track to `&dyn Any`.
    fn as_any(&self) -> &dyn std::any::Any;
    /// Upcast this track to `&mut dyn Any`.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// A strongly typed compiled track containing contiguous, sorted, non-overlapping clips.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledTrack<T: Interpolate> {
    /// Track identifier name.
    pub name: String,
    /// Flat, sorted sequence of clips for this track.
    pub clips: Vec<Clip<T>>,
    /// Total duration of this track (furthest clip `end_time`).
    pub duration: Duration,
}

impl<T: Interpolate> CompiledTrack<T> {
    /// Create a new compiled track from sorted clips.
    pub fn new(name: String, clips: Vec<Clip<T>>, duration: Duration) -> Self {
        Self {
            name,
            clips,
            duration,
        }
    }

    /// Return the name of this track.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the total duration of this track.
    #[inline]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Return all clips on this track.
    #[inline]
    pub fn clips(&self) -> &[Clip<T>] {
        &self.clips
    }

    /// Return a reference to the clip at index `idx`.
    #[inline]
    pub fn clip(&self, idx: usize) -> Option<&Clip<T>> {
        self.clips.get(idx)
    }

    /// Return the total number of clips on this track.
    #[inline]
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// Return `true` if this track has no clips.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }
}

impl<T: Interpolate + Send + Sync + 'static> AnyCompiledTrack for CompiledTrack<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn clip_count(&self) -> usize {
        self.clips.len()
    }

    fn is_spanned_at(&self, idx: usize) -> bool {
        self.clips.get(idx).map(|c| c.is_spanned()).unwrap_or(false)
    }

    fn clip_interval(&self, idx: usize) -> Option<(Duration, Duration)> {
        self.clips.get(idx).map(|c| (c.start_time, c.end_time()))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Fully compiled, validated multi-track timeline ready for sampling and playback.
pub struct CompiledTimeline {
    pub(crate) tracks: HashMap<String, Box<dyn AnyCompiledTrack>>,
    pub(crate) labels: HashMap<String, Duration>,
    pub(crate) duration: Duration,
    pub(crate) track_names: Vec<String>,
}

impl CompiledTimeline {
    /// Total duration of the compiled timeline across all tracks and labels.
    #[inline]
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Return a reference to the label dictionary.
    #[inline]
    pub fn labels(&self) -> &HashMap<String, Duration> {
        &self.labels
    }

    /// Query the absolute timestamp of a label, if registered.
    #[inline]
    pub fn label(&self, name: &str) -> Option<Duration> {
        self.labels.get(name).copied()
    }

    /// Return track names in insertion order.
    #[inline]
    pub fn track_names(&self) -> &[String] {
        &self.track_names
    }

    /// Return `true` if a track with `name` exists in this timeline.
    #[inline]
    pub fn has_track(&self, name: &str) -> bool {
        self.tracks.contains_key(name)
    }

    /// Return the total number of tracks in this timeline.
    #[inline]
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Query a type-erased track reference by name.
    #[inline]
    pub fn any_track(&self, name: &str) -> Option<&dyn AnyCompiledTrack> {
        self.tracks.get(name).map(|b| &**b)
    }

    /// Query a strongly typed track by name.
    ///
    /// # Errors
    /// Returns `TimelineError::TrackNotFound` if the track name does not exist
    /// or if the channel type `T` does not match the track's compiled type.
    pub fn track<T: Interpolate + 'static>(
        &self,
        name: &str,
    ) -> Result<&CompiledTrack<T>, TimelineError> {
        let boxed = self
            .tracks
            .get(name)
            .ok_or_else(|| TimelineError::TrackNotFound(name.to_string()))?;
        boxed
            .as_any()
            .downcast_ref::<CompiledTrack<T>>()
            .ok_or_else(|| {
                TimelineError::TrackNotFound(format!(
                    "track '{name}' exists but channel type does not match requested type"
                ))
            })
    }
}

impl std::fmt::Debug for CompiledTimeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledTimeline")
            .field("duration", &self.duration)
            .field("labels", &self.labels)
            .field("track_names", &self.track_names)
            .finish()
    }
}
