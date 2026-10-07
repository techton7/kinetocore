//! Multi-track timeline sequencing engine with headless authoring IR and interval compiler.

pub mod builder;
pub mod clip;
pub mod compiled;
pub mod error;
pub mod keyframe;
pub mod position;

pub use builder::{TimelineBuilder, TrackBuilder};
pub use clip::{Clip, ClipKind, InstantClipKind, SpannedClipKind};
pub use compiled::{AnyCompiledTrack, CompiledTimeline, CompiledTrack};
pub use error::TimelineError;
pub use keyframe::{validate_keyframes, Keyframe};
pub use position::{OffsetSign, Position, SignedDuration};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::ease::Ease;

    #[test]
    fn test_position_parsing_all_variants() {
        // Empty
        assert_eq!(Position::parse("").unwrap(), Position::End);

        // Absolute
        assert_eq!(
            Position::parse("1.5").unwrap(),
            Position::Absolute(Duration::from_secs_f64(1.5))
        );
        assert_eq!(
            Position::parse("1.5s").unwrap(),
            Position::Absolute(Duration::from_secs_f64(1.5))
        );
        assert_eq!(
            Position::parse("500ms").unwrap(),
            Position::Absolute(Duration::from_millis(500))
        );

        // Relative offsets
        assert_eq!(
            Position::parse("+=0.2s").unwrap(),
            Position::Offset(SignedDuration::positive(Duration::from_millis(200)))
        );
        assert_eq!(
            Position::parse("+=200ms").unwrap(),
            Position::Offset(SignedDuration::positive(Duration::from_millis(200)))
        );
        assert_eq!(
            Position::parse("-=0.1s").unwrap(),
            Position::Offset(SignedDuration::negative(Duration::from_millis(100)))
        );
        assert_eq!(
            Position::parse("-=100ms").unwrap(),
            Position::Offset(SignedDuration::negative(Duration::from_millis(100)))
        );

        // Recent clips
        assert_eq!(Position::parse("<").unwrap(), Position::RecentStart);
        assert_eq!(Position::parse(">").unwrap(), Position::RecentEnd);
        assert_eq!(
            Position::parse("<+=0.2s").unwrap(),
            Position::RecentStartOffset(SignedDuration::positive(Duration::from_millis(200)))
        );
        assert_eq!(
            Position::parse("<-=0.1s").unwrap(),
            Position::RecentStartOffset(SignedDuration::negative(Duration::from_millis(100)))
        );
        assert_eq!(
            Position::parse(">+=0.5s").unwrap(),
            Position::RecentEndOffset(SignedDuration::positive(Duration::from_millis(500)))
        );
        assert_eq!(
            Position::parse(">-=-0.2s").unwrap(),
            Position::RecentEndOffset(SignedDuration::negative(Duration::from_millis(200)))
        );

        // Labels and label offsets
        assert_eq!(
            Position::parse("intro").unwrap(),
            Position::Label("intro".to_string())
        );
        assert_eq!(
            Position::parse("intro+=0.3s").unwrap(),
            Position::LabelOffset(
                "intro".to_string(),
                SignedDuration::positive(Duration::from_millis(300))
            )
        );
        assert_eq!(
            Position::parse("intro-=0.2s").unwrap(),
            Position::LabelOffset(
                "intro".to_string(),
                SignedDuration::negative(Duration::from_millis(200))
            )
        );
    }

    #[test]
    fn test_label_resolution_and_relative_offsets() {
        let mut builder = TimelineBuilder::new();
        builder
            .tween(
                "x",
                0.0f64,
                10.0f64,
                Duration::from_secs(1),
                Ease::Linear,
                Position::End,
            )
            .add_label("mid", Position::RecentEnd)
            .add_label(
                "mid_plus",
                Position::LabelOffset(
                    "mid".to_string(),
                    SignedDuration::positive(Duration::from_millis(500)),
                ),
            )
            .tween(
                "y",
                0.0f64,
                50.0f64,
                Duration::from_secs(1),
                Ease::QuadInOut,
                Position::Label("mid_plus".to_string()),
            );

        let compiled = builder.compile().expect("compilation should succeed");
        assert_eq!(compiled.label("mid"), Some(Duration::from_secs(1)));
        assert_eq!(
            compiled.label("mid_plus"),
            Some(Duration::from_millis(1500))
        );
        assert_eq!(compiled.duration(), Duration::from_millis(2500));

        let track_y = compiled.track::<f64>("y").expect("track y should exist");
        assert_eq!(track_y.clips().len(), 1);
        assert_eq!(track_y.clips()[0].start_time, Duration::from_millis(1500));
        assert_eq!(track_y.clips()[0].end_time(), Duration::from_millis(2500));
    }

    #[test]
    fn test_unknown_label_error() {
        let mut builder = TimelineBuilder::new();
        builder.tween(
            "x",
            0.0f64,
            10.0f64,
            Duration::from_secs(1),
            Ease::Linear,
            Position::Label("non_existent".to_string()),
        );

        let err = builder.compile().unwrap_err();
        assert_eq!(
            err,
            TimelineError::UnknownLabel("non_existent".to_string())
        );
    }

    #[test]
    fn test_keyframe_monotonicity_and_boundary_validation() {
        let valid_kfs = vec![
            Keyframe::linear(0.0, 0.0f32),
            Keyframe::new(0.5, 50.0f32, Ease::CubicIn),
            Keyframe::linear(1.0, 100.0f32),
        ];
        assert!(validate_keyframes(&valid_kfs).is_ok());

        // First != 0.0
        let invalid_start = vec![
            Keyframe::linear(0.1, 0.0f32),
            Keyframe::linear(1.0, 100.0f32),
        ];
        assert!(matches!(
            validate_keyframes(&invalid_start),
            Err(TimelineError::InvalidKeyframe(_))
        ));

        // Last != 1.0
        let invalid_end = vec![
            Keyframe::linear(0.0, 0.0f32),
            Keyframe::linear(0.8, 100.0f32),
        ];
        assert!(matches!(
            validate_keyframes(&invalid_end),
            Err(TimelineError::InvalidKeyframe(_))
        ));

        // Non-monotonic
        let non_monotonic = vec![
            Keyframe::linear(0.0, 0.0f32),
            Keyframe::linear(0.6, 50.0f32),
            Keyframe::linear(0.4, 70.0f32),
            Keyframe::linear(1.0, 100.0f32),
        ];
        assert!(matches!(
            validate_keyframes(&non_monotonic),
            Err(TimelineError::InvalidKeyframe(_))
        ));

        // Duplicate
        let duplicate = vec![
            Keyframe::linear(0.0, 0.0f32),
            Keyframe::linear(0.5, 50.0f32),
            Keyframe::linear(0.5, 70.0f32),
            Keyframe::linear(1.0, 100.0f32),
        ];
        assert!(matches!(
            validate_keyframes(&duplicate),
            Err(TimelineError::InvalidKeyframe(_))
        ));

        // When supplied to TimelineBuilder, non-monotonic keyframes fail compile
        let mut builder = TimelineBuilder::new();
        builder.keyframes(
            "k",
            non_monotonic,
            Duration::from_secs(1),
            Position::Absolute(Duration::ZERO),
        );
        assert!(matches!(
            builder.compile(),
            Err(TimelineError::InvalidKeyframe(_))
        ));
    }

    #[test]
    fn test_overlap_rejection_error_on_same_track() {
        let mut builder = TimelineBuilder::new();
        builder
            .tween(
                "pos",
                0.0f64,
                10.0f64,
                Duration::from_secs(1),
                Ease::Linear,
                Position::Absolute(Duration::ZERO),
            )
            .tween(
                "pos",
                10.0f64,
                20.0f64,
                Duration::from_secs(1),
                Ease::Linear,
                Position::Absolute(Duration::from_millis(500)), // Overlaps with [0s, 1s)
            );

        let err = builder.compile().unwrap_err();
        match err {
            TimelineError::TrackIntervalOverlap {
                track,
                start1,
                end1,
                start2,
                end2,
            } => {
                assert_eq!(track, "pos");
                assert_eq!(start1, Duration::ZERO);
                assert_eq!(end1, Duration::from_secs(1));
                assert_eq!(start2, Duration::from_millis(500));
                assert_eq!(end2, Duration::from_millis(1500));
            }
            other => panic!("Expected TrackIntervalOverlap, got {other:?}"),
        }
    }

    #[test]
    fn test_abutting_contiguous_clips_compilation_success() {
        let mut builder = TimelineBuilder::new();
        builder
            .tween(
                "pos",
                0.0f64,
                10.0f64,
                Duration::from_secs(1),
                Ease::Linear,
                Position::Absolute(Duration::ZERO),
            )
            .tween(
                "pos",
                10.0f64,
                20.0f64,
                Duration::from_secs(1),
                Ease::Linear,
                Position::Absolute(Duration::from_secs(1)), // Abutting: start2 == end1 == 1.0s
            );

        let compiled = builder.compile().expect("abutting clips must compile cleanly");
        let track = compiled.track::<f64>("pos").unwrap();
        assert_eq!(track.clips().len(), 2);
        assert_eq!(track.clips()[0].end_time(), Duration::from_secs(1));
        assert_eq!(track.clips()[1].start_time, Duration::from_secs(1));
    }

    #[test]
    fn test_non_overlapping_multi_track_compilation_success() {
        let mut builder = TimelineBuilder::new();
        // Tracks "x", "y", "color" run concurrently
        builder
            .track::<f64>("x", |t| {
                t.tween(
                    0.0,
                    100.0,
                    Duration::from_secs(2),
                    Ease::QuadInOut,
                    Position::Absolute(Duration::ZERO),
                );
            })
            .track::<f64>("y", |t| {
                t.tween(
                    0.0,
                    50.0,
                    Duration::from_secs(2),
                    Ease::QuadInOut,
                    Position::Absolute(Duration::ZERO), // Same interval, different track
                );
            })
            .track::<[f32; 4]>("color", |t| {
                t.set([1.0, 0.0, 0.0, 1.0], Position::Absolute(Duration::ZERO))
                    .tween(
                        [1.0, 0.0, 0.0, 1.0],
                        [0.0, 1.0, 0.0, 1.0],
                        Duration::from_secs(1),
                        Ease::Linear,
                        Position::Absolute(Duration::from_millis(500)),
                    );
            });

        let compiled = builder
            .compile()
            .expect("multi-track compilation should succeed");
        assert_eq!(compiled.track_names().len(), 3);
        assert_eq!(compiled.duration(), Duration::from_secs(2));

        let track_x = compiled.track::<f64>("x").unwrap();
        assert_eq!(track_x.clips().len(), 1);

        let track_color = compiled.track::<[f32; 4]>("color").unwrap();
        assert_eq!(track_color.clips().len(), 2);
        assert!(track_color.clips()[0].is_instant());
        assert!(track_color.clips()[1].is_spanned());
    }

    #[test]
    fn test_empty_timeline_error() {
        let builder = TimelineBuilder::new();
        let err = builder.compile().unwrap_err();
        assert_eq!(err, TimelineError::EmptyTimeline);
    }
}
