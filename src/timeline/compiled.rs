//! Compiled timeline representations with flat, validated, allocation-free intervals.

use std::collections::HashMap;
use std::time::Duration;

use crate::interpolate::Interpolate;
use crate::timeline::clip::Clip;
use crate::timeline::error::TimelineError;
use crate::timeline::sampler::{BoundaryPolicy, TrackSampler};

/// Type-erased trait for querying compiled tracks without knowing their target generic type.
pub trait AnyCompiledTrack: TrackSampler {
    /// Return the total number of clips on the track.
    fn clip_count(&self) -> usize;
    /// Returns `true` if the clip at `idx` is spanned.
    fn is_spanned_at(&self, idx: usize) -> bool;
    /// Return the `(start_time, end_time)` interval of the clip at `idx`.
    fn clip_interval(&self, idx: usize) -> Option<(Duration, Duration)>;
    /// Upcast this track to `&mut dyn Any`.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    /// Clone this erased track into a heap box.
    fn clone_box(&self) -> Box<dyn AnyCompiledTrack>;
}

impl Clone for Box<dyn AnyCompiledTrack> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// A strongly typed compiled track containing contiguous, sorted, non-overlapping clips.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledTrack<T: Interpolate> {
    /// Track identifier name.
    pub name: String,
    /// Initial value held during pre-roll ($t < \text{first\_clip.start\_time}$).
    pub initial_value: T,
    /// Flat, sorted sequence of clips for this track.
    pub clips: Vec<Clip<T>>,
    /// Total duration of this track (furthest clip `end_time`).
    pub duration: Duration,
}

impl<T: Interpolate> CompiledTrack<T> {
    /// Create a new compiled track from sorted clips and an initial value.
    pub fn new(name: String, initial_value: T, clips: Vec<Clip<T>>, duration: Duration) -> Self {
        Self {
            name,
            initial_value,
            clips,
            duration,
        }
    }

    /// Return the name of this track.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the initial pre-roll value of this track.
    #[inline]
    pub fn initial_value(&self) -> &T {
        &self.initial_value
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

    /// Sample this track at `time` using deterministic boundary and interval policies:
    ///
    /// 1. **Pre-roll** ($t < t_{\text{start}}$): returns `self.initial_value`.
    /// 2. **Active clip** ($t \in [t_{\text{start}}, t_{\text{end}}]$): evaluated per clip kind.
    /// 3. **Inter-clip gap** ($t_{\text{prev.end}} < t < t_{\text{next.start}}$): forward-fills preceding terminal value.
    /// 4. **Post-roll** ($t > t_{\text{end}}$): holds last clip's terminal value.
    pub fn sample_at(&self, time: Duration) -> T {
        if self.clips.is_empty() {
            return self.initial_value.clone();
        }

        // 1. Pre-roll
        if time < self.clips[0].start_time {
            return self.initial_value.clone();
        }

        // 4. Post-roll
        let last_clip = &self.clips[self.clips.len() - 1];
        if time > last_clip.end_time() {
            return last_clip.terminal_value();
        }

        // Find clips starting at or before `time`
        let idx = self.clips.partition_point(|c| c.start_time <= time);
        if idx == 0 {
            return self.initial_value.clone();
        }

        let latest = &self.clips[idx - 1];

        if latest.is_spanned() {
            if time <= latest.end_time() {
                // Active spanned clip
                latest.sample_at(time)
            } else {
                // Gap between clips: forward-fill latest terminal value
                latest.terminal_value()
            }
        } else {
            // Instant clip
            if latest.start_time == time {
                latest.terminal_value()
            } else {
                // Check if an earlier spanned clip is still actively covering `time`
                let active_spanned = self.clips[..idx]
                    .iter()
                    .rev()
                    .find(|c| c.is_spanned() && time <= c.end_time());

                if let Some(spanned) = active_spanned {
                    spanned.sample_at(time)
                } else {
                    latest.terminal_value()
                }
            }
        }
    }

    /// Query the boundary policy applying to `time`, or `None` if an active clip interval covers `time`.
    pub fn boundary_at(&self, time: Duration) -> Option<BoundaryPolicy> {
        if self.clips.is_empty() {
            return Some(BoundaryPolicy::HoldInitial);
        }

        if time < self.clips[0].start_time {
            return Some(BoundaryPolicy::HoldInitial);
        }

        let last_clip = &self.clips[self.clips.len() - 1];
        if time > last_clip.end_time() {
            return Some(BoundaryPolicy::HoldTerminal);
        }

        let idx = self.clips.partition_point(|c| c.start_time <= time);
        if idx > 0 {
            let latest = &self.clips[idx - 1];
            if latest.is_spanned() && time <= latest.end_time() {
                return None;
            }
            if latest.is_instant() && latest.start_time == time {
                return None;
            }
            let active_spanned = self.clips[..idx]
                .iter()
                .rev()
                .any(|c| c.is_spanned() && time <= c.end_time());
            if active_spanned {
                return None;
            }
            if idx < self.clips.len() && time < self.clips[idx].start_time {
                return Some(BoundaryPolicy::ForwardFill);
            }
        }

        None
    }
}

impl<T: Interpolate + Send + Sync + 'static> TrackSampler for CompiledTrack<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl<T: Interpolate + Send + Sync + 'static> AnyCompiledTrack for CompiledTrack<T> {
    fn clip_count(&self) -> usize {
        self.clips.len()
    }

    fn is_spanned_at(&self, idx: usize) -> bool {
        self.clips.get(idx).map(|c| c.is_spanned()).unwrap_or(false)
    }

    fn clip_interval(&self, idx: usize) -> Option<(Duration, Duration)> {
        self.clips.get(idx).map(|c| (c.start_time, c.end_time()))
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn AnyCompiledTrack> {
        Box::new(self.clone())
    }
}

/// Fully compiled, validated multi-track timeline ready for sampling and playback.
#[derive(Clone)]
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

    /// Query a type-erased [`TrackSampler`] by name.
    #[inline]
    pub fn track_sampler(&self, name: &str) -> Option<&dyn TrackSampler> {
        self.tracks.get(name).map(|b| b.as_ref() as &dyn TrackSampler)
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

    /// Sample a strongly typed track by name at `time`.
    ///
    /// Evaluates the track at `time` using deterministic boundary and interval policies.
    /// Returns `None` if the track name does not exist or if `T` does not match
    /// the track's channel type.
    ///
    /// This method performs zero heap allocations.
    #[inline]
    pub fn sample_track<T: Interpolate + 'static>(&self, name: &str, time: Duration) -> Option<T> {
        let track = self.track::<T>(name).ok()?;
        Some(track.sample_at(time))
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
