//! Declarative authoring IR and compiler resolving absolute intervals with overlap rejection.

use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;

use crate::ease::Ease;
use crate::interpolate::Interpolate;
use crate::timeline::clip::{Clip, ClipKind, InstantClipKind, SpannedClipKind};
use crate::timeline::compiled::{AnyCompiledTrack, CompiledTimeline, CompiledTrack};
use crate::timeline::error::TimelineError;
use crate::timeline::keyframe::{validate_keyframes, Keyframe};
use crate::timeline::position::Position;

trait ErasedClipAuthoring: Send + Sync {
    fn track_name(&self) -> &str;
    fn position(&self) -> &Position;
    fn duration(&self) -> Duration;
    fn validate(&self) -> Result<(), TimelineError>;
    fn track_type_id(&self) -> TypeId;
    fn track_type_name(&self) -> &'static str;
    fn create_track_builder(&self) -> Box<dyn ErasedTrackBuilder>;
    fn build_clip(
        self: Box<Self>,
        id: usize,
        start_time: Duration,
    ) -> Box<dyn std::any::Any + Send + Sync>;
}

struct TypedClipAuthoring<T: Interpolate + Send + Sync + 'static> {
    track: String,
    position: Position,
    duration: Duration,
    kind: ClipKind<T>,
}

impl<T: Interpolate + Send + Sync + 'static> ErasedClipAuthoring for TypedClipAuthoring<T> {
    fn track_name(&self) -> &str {
        &self.track
    }

    fn position(&self) -> &Position {
        &self.position
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn validate(&self) -> Result<(), TimelineError> {
        match &self.kind {
            ClipKind::Instant(_) => Ok(()),
            ClipKind::Spanned(spanned) => {
                if self.duration == Duration::ZERO {
                    return Err(TimelineError::InvalidPosition(
                        "spanned clip duration must be greater than zero".into(),
                    ));
                }
                if let SpannedClipKind::Keyframes { keyframes } = spanned {
                    validate_keyframes(keyframes)?;
                }
                Ok(())
            }
        }
    }

    fn track_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn track_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn create_track_builder(&self) -> Box<dyn ErasedTrackBuilder> {
        Box::new(TypedTrackBuilder::<T> {
            name: self.track.clone(),
            initial_value: None,
            clips: Vec::new(),
        })
    }

    fn build_clip(
        self: Box<Self>,
        id: usize,
        start_time: Duration,
    ) -> Box<dyn std::any::Any + Send + Sync> {
        let clip = Clip {
            id,
            start_time,
            duration: self.duration,
            kind: self.kind,
        };
        Box::new(clip)
    }
}

trait ErasedTrackBuilder: Send + Sync {
    fn track_type_id(&self) -> TypeId;
    fn track_type_name(&self) -> &'static str;
    fn set_erased_initial_value(&mut self, val_box: Box<dyn std::any::Any + Send + Sync>);
    fn add_erased_clip(&mut self, clip_box: Box<dyn std::any::Any + Send + Sync>);
    fn compile_track(
        self: Box<Self>,
        track_duration: Duration,
    ) -> Result<Box<dyn AnyCompiledTrack>, TimelineError>;
}

struct TypedTrackBuilder<T: Interpolate + Send + Sync + 'static> {
    name: String,
    initial_value: Option<T>,
    clips: Vec<Clip<T>>,
}

impl<T: Interpolate + Send + Sync + 'static> ErasedTrackBuilder for TypedTrackBuilder<T> {
    fn track_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn track_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn set_erased_initial_value(&mut self, val_box: Box<dyn std::any::Any + Send + Sync>) {
        if let Ok(val) = val_box.downcast::<T>() {
            self.initial_value = Some(*val);
        }
    }

    fn add_erased_clip(&mut self, clip_box: Box<dyn std::any::Any + Send + Sync>) {
        let clip = *clip_box
            .downcast::<Clip<T>>()
            .expect("downcast clip type match");
        self.clips.push(clip);
    }

    fn compile_track(
        mut self: Box<Self>,
        track_duration: Duration,
    ) -> Result<Box<dyn AnyCompiledTrack>, TimelineError> {
        // Sort clips deterministically by start_time, breaking ties by insertion ID
        self.clips
            .sort_by(|a, b| a.start_time.cmp(&b.start_time).then(a.id.cmp(&b.id)));

        // Single-track overlap check: reject if two spanned clips overlap
        let mut prev_spanned: Option<&Clip<T>> = None;
        for curr in &self.clips {
            if curr.is_spanned() {
                if let Some(prev) = prev_spanned {
                    if curr.start_time < prev.end_time() {
                        return Err(TimelineError::TrackIntervalOverlap {
                            track: self.name.clone(),
                            start1: prev.start_time,
                            end1: prev.end_time(),
                            start2: curr.start_time,
                            end2: curr.end_time(),
                        });
                    }
                    if curr.end_time() > prev.end_time() {
                        prev_spanned = Some(curr);
                    }
                } else {
                    prev_spanned = Some(curr);
                }
            }
        }

        let initial_value = match self.initial_value {
            Some(v) => v,
            None => {
                self.clips
                    .first()
                    .map(|c| c.start_value())
                    .ok_or(TimelineError::EmptyTimeline)?
            }
        };

        let compiled = CompiledTrack::new(self.name, initial_value, self.clips, track_duration);
        Ok(Box::new(compiled))
    }
}

enum AuthoringCommand {
    Label {
        name: String,
        position: Position,
    },
    Clip(Box<dyn ErasedClipAuthoring>),
}

/// Declarative builder for authoring multi-track timeline choreographies.
#[derive(Default)]
pub struct TimelineBuilder {
    commands: Vec<AuthoringCommand>,
    initial_values: HashMap<String, (Box<dyn std::any::Any + Send + Sync>, TypeId, &'static str)>,
}

impl TimelineBuilder {
    /// Create a new, empty timeline builder.
    #[inline]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            initial_values: HashMap::new(),
        }
    }

    /// Explicitly set an initial value for a track.
    pub fn set_initial_value<T: Interpolate + Send + Sync + 'static>(
        &mut self,
        track: impl Into<String>,
        value: T,
    ) -> &mut Self {
        self.initial_values.insert(
            track.into(),
            (
                Box::new(value),
                TypeId::of::<T>(),
                std::any::type_name::<T>(),
            ),
        );
        self
    }

    /// Add a named synchronization label at `position`.
    pub fn add_label(
        &mut self,
        name: impl Into<String>,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.commands.push(AuthoringCommand::Label {
            name: name.into(),
            position: position.into(),
        });
        self
    }

    /// Alias for `add_label`.
    #[inline]
    pub fn label(&mut self, name: impl Into<String>, position: impl Into<Position>) -> &mut Self {
        self.add_label(name, position)
    }

    /// Add a generic clip to a track at `position`.
    pub fn add_clip<T: Interpolate + Send + Sync + 'static>(
        &mut self,
        track: impl Into<String>,
        position: impl Into<Position>,
        duration: Duration,
        kind: ClipKind<T>,
    ) -> &mut Self {
        let authoring = TypedClipAuthoring {
            track: track.into(),
            position: position.into(),
            duration,
            kind,
        };
        self.commands
            .push(AuthoringCommand::Clip(Box::new(authoring)));
        self
    }

    /// Add an instant `Set` clip updating `track` to `value` at `position`.
    pub fn set<T: Interpolate + Send + Sync + 'static>(
        &mut self,
        track: impl Into<String>,
        value: T,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.add_clip(
            track,
            position,
            Duration::ZERO,
            ClipKind::Instant(InstantClipKind::Set { value }),
        )
    }

    /// Add a continuous `Tween` clip interpolating between `from` and `to` over `duration`.
    pub fn tween<T: Interpolate + Send + Sync + 'static>(
        &mut self,
        track: impl Into<String>,
        from: T,
        to: T,
        duration: Duration,
        ease: Ease,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.add_clip(
            track,
            position,
            duration,
            ClipKind::Spanned(SpannedClipKind::Tween { from, to, ease }),
        )
    }

    /// Add a `Hold` clip maintaining `value` constant over `duration`.
    pub fn hold<T: Interpolate + Send + Sync + 'static>(
        &mut self,
        track: impl Into<String>,
        value: T,
        duration: Duration,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.add_clip(
            track,
            position,
            duration,
            ClipKind::Spanned(SpannedClipKind::Hold { value }),
        )
    }

    /// Add a multi-keyframe clip over `duration`.
    pub fn keyframes<T: Interpolate + Send + Sync + 'static>(
        &mut self,
        track: impl Into<String>,
        keyframes: Vec<Keyframe<T>>,
        duration: Duration,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.add_clip(
            track,
            position,
            duration,
            ClipKind::Spanned(SpannedClipKind::Keyframes { keyframes }),
        )
    }

    /// Scope clip authoring to a specific typed track with an explicit initial value.
    pub fn track_with_initial<T: Interpolate + Send + Sync + 'static>(
        &mut self,
        name: impl Into<String>,
        initial_value: T,
        f: impl FnOnce(&mut TrackBuilder<T>),
    ) -> &mut Self {
        let name_str = name.into();
        self.set_initial_value(name_str.clone(), initial_value);
        let mut track_builder = TrackBuilder {
            builder: self,
            track_name: name_str,
            _marker: PhantomData,
        };
        f(&mut track_builder);
        self
    }

    /// Scope clip authoring to a specific typed track.
    pub fn track<T: Interpolate + Send + Sync + 'static>(
        &mut self,
        name: impl Into<String>,
        f: impl FnOnce(&mut TrackBuilder<T>),
    ) -> &mut Self {
        let mut track_builder = TrackBuilder {
            builder: self,
            track_name: name.into(),
            _marker: PhantomData,
        };
        f(&mut track_builder);
        self
    }

    /// Compile authoring commands into a validated, flat `CompiledTimeline`.
    ///
    /// # Resolution Steps
    /// 1. Resolves all positions in sequential insertion order.
    /// 2. Tracks `recent_clip: Option<(Duration, Duration)>` and `timeline_duration`.
    /// 3. Validates label references and keyframe sequences.
    /// 4. Groups clips by track and sorts each track by `start_time`.
    /// 5. Validates non-overlapping intervals for spanned clips on the same track.
    ///
    /// # Errors
    /// Returns `TimelineError::TrackIntervalOverlap` if two spanned clips on the same track overlap.
    /// Returns `TimelineError::UnknownLabel` if a position references an unknown label.
    /// Returns `TimelineError::EmptyTimeline` if no tracks or clips are registered.
    pub fn compile(self) -> Result<CompiledTimeline, TimelineError> {
        let mut labels: HashMap<String, Duration> = HashMap::new();
        let mut timeline_duration = Duration::ZERO;
        let mut recent_clip: Option<(Duration, Duration)> = None;

        let mut track_builders: HashMap<String, Box<dyn ErasedTrackBuilder>> = HashMap::new();
        let mut track_durations: HashMap<String, Duration> = HashMap::new();
        let mut track_names: Vec<String> = Vec::new();
        let mut next_clip_id = 0;
        let mut total_clips = 0;

        for cmd in self.commands {
            match cmd {
                AuthoringCommand::Label { name, position } => {
                    let label_time = resolve_position(
                        &position,
                        &labels,
                        recent_clip,
                        timeline_duration,
                    )?;
                    labels.insert(name, label_time);
                    if label_time > timeline_duration {
                        timeline_duration = label_time;
                    }
                }
                AuthoringCommand::Clip(authoring_clip) => {
                    authoring_clip.validate()?;

                    let track_name = authoring_clip.track_name().to_string();
                    let position = authoring_clip.position().clone();
                    let duration = authoring_clip.duration();

                    let start_time = resolve_position(
                        &position,
                        &labels,
                        recent_clip,
                        timeline_duration,
                    )?;
                    let end_time = start_time + duration;

                    recent_clip = Some((start_time, duration));
                    if end_time > timeline_duration {
                        timeline_duration = end_time;
                    }

                    let current_track_dur = track_durations.entry(track_name.clone()).or_insert(Duration::ZERO);
                    if end_time > *current_track_dur {
                        *current_track_dur = end_time;
                    }

                    let builder = match track_builders.get_mut(&track_name) {
                        Some(existing) => {
                            if existing.track_type_id() != authoring_clip.track_type_id() {
                                return Err(TimelineError::InvalidPosition(format!(
                                    "track '{track_name}' channel type mismatch: expected {}, found {}",
                                    existing.track_type_name(),
                                    authoring_clip.track_type_name()
                                )));
                            }
                            existing
                        }
                        None => {
                            let new_builder = authoring_clip.create_track_builder();
                            track_names.push(track_name.clone());
                            track_builders.insert(track_name.clone(), new_builder);
                            track_builders.get_mut(&track_name).unwrap()
                        }
                    };

                    let clip_id = next_clip_id;
                    next_clip_id += 1;
                    total_clips += 1;

                    let compiled_clip = authoring_clip.build_clip(clip_id, start_time);
                    builder.add_erased_clip(compiled_clip);
                }
            }
        }

        if total_clips == 0 {
            return Err(TimelineError::EmptyTimeline);
        }

        for (track_name, (initial_box, type_id, type_name)) in self.initial_values {
            if let Some(builder) = track_builders.get_mut(&track_name) {
                if builder.track_type_id() != type_id {
                    return Err(TimelineError::InvalidPosition(format!(
                        "track '{track_name}' channel type mismatch: expected {}, found {}",
                        builder.track_type_name(),
                        type_name
                    )));
                }
                builder.set_erased_initial_value(initial_box);
            }
        }

        let mut compiled_tracks: HashMap<String, Box<dyn AnyCompiledTrack>> = HashMap::new();
        for (name, builder) in track_builders {
            let dur = track_durations.get(&name).copied().unwrap_or(Duration::ZERO);
            let compiled = builder.compile_track(dur)?;
            compiled_tracks.insert(name, compiled);
        }

        Ok(CompiledTimeline {
            tracks: compiled_tracks,
            labels,
            duration: timeline_duration,
            track_names,
        })
    }
}

/// Helper for building sequential clips on a specific track channel.
pub struct TrackBuilder<'a, T: Interpolate + Send + Sync + 'static> {
    builder: &'a mut TimelineBuilder,
    track_name: String,
    _marker: PhantomData<T>,
}

impl<'a, T: Interpolate + Send + Sync + 'static> TrackBuilder<'a, T> {
    /// Explicitly set an initial value for this track.
    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.builder.set_initial_value(self.track_name.clone(), value);
        self
    }

    /// Add an instant `Set` clip to this track.
    pub fn set(&mut self, value: T, position: impl Into<Position>) -> &mut Self {
        self.builder.set(self.track_name.clone(), value, position);
        self
    }

    /// Add a continuous `Tween` clip to this track.
    pub fn tween(
        &mut self,
        from: T,
        to: T,
        duration: Duration,
        ease: Ease,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.builder
            .tween(self.track_name.clone(), from, to, duration, ease, position);
        self
    }

    /// Alias for `tween`.
    #[inline]
    pub fn to(
        &mut self,
        from: T,
        to: T,
        duration: Duration,
        ease: Ease,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.tween(from, to, duration, ease, position)
    }

    /// Add a `Hold` clip to this track.
    pub fn hold(
        &mut self,
        value: T,
        duration: Duration,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.builder.hold(self.track_name.clone(), value, duration, position);
        self
    }

    /// Add a multi-keyframe clip to this track.
    pub fn keyframes(
        &mut self,
        keyframes: Vec<Keyframe<T>>,
        duration: Duration,
        position: impl Into<Position>,
    ) -> &mut Self {
        self.builder
            .keyframes(self.track_name.clone(), keyframes, duration, position);
        self
    }

    /// Add a raw clip to this track.
    pub fn add_clip(
        &mut self,
        position: impl Into<Position>,
        duration: Duration,
        kind: ClipKind<T>,
    ) -> &mut Self {
        self.builder
            .add_clip(self.track_name.clone(), position, duration, kind);
        self
    }
}

fn resolve_position(
    position: &Position,
    labels: &HashMap<String, Duration>,
    recent_clip: Option<(Duration, Duration)>,
    timeline_duration: Duration,
) -> Result<Duration, TimelineError> {
    match position {
        Position::End => Ok(timeline_duration),
        Position::Absolute(d) => Ok(*d),
        Position::Offset(signed) => Ok(signed.apply_to(timeline_duration)),
        Position::RecentStart => {
            let start = recent_clip.map(|(s, _)| s).unwrap_or(Duration::ZERO);
            Ok(start)
        }
        Position::RecentEnd => {
            let end = recent_clip
                .map(|(s, d)| s + d)
                .unwrap_or(timeline_duration);
            Ok(end)
        }
        Position::RecentStartOffset(signed) => {
            let start = recent_clip.map(|(s, _)| s).unwrap_or(Duration::ZERO);
            Ok(signed.apply_to(start))
        }
        Position::RecentEndOffset(signed) => {
            let end = recent_clip
                .map(|(s, d)| s + d)
                .unwrap_or(timeline_duration);
            Ok(signed.apply_to(end))
        }
        Position::Label(name) => labels
            .get(name)
            .copied()
            .ok_or_else(|| TimelineError::UnknownLabel(name.clone())),
        Position::LabelOffset(name, signed) => {
            let base = labels
                .get(name)
                .copied()
                .ok_or_else(|| TimelineError::UnknownLabel(name.clone()))?;
            Ok(signed.apply_to(base))
        }
    }
}
