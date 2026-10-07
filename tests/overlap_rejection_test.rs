use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_same_track_overlap_rejection() {
    let mut builder = TimelineBuilder::new();
    builder
        .tween(
            "scale",
            1.0f32,
            2.0f32,
            Duration::from_secs(2),
            Ease::Linear,
            Position::Absolute(Duration::from_secs(1)), // [1.0s, 3.0s]
        )
        .tween(
            "scale",
            2.0f32,
            1.5f32,
            Duration::from_secs(1),
            Ease::Linear,
            Position::Absolute(Duration::from_millis(2500)), // [2.5s, 3.5s] -> overlaps!
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
            assert_eq!(track, "scale");
            assert_eq!(start1, Duration::from_secs(1));
            assert_eq!(end1, Duration::from_secs(3));
            assert_eq!(start2, Duration::from_millis(2500));
            assert_eq!(end2, Duration::from_millis(3500));
        }
        other => panic!("Expected TrackIntervalOverlap, got {other:?}"),
    }
}

#[test]
fn test_abutting_same_track_intervals_are_valid() {
    let mut builder = TimelineBuilder::new();
    builder
        .tween(
            "alpha",
            0.0f32,
            0.5f32,
            Duration::from_secs(1),
            Ease::Linear,
            Position::Absolute(Duration::ZERO), // [0s, 1s]
        )
        .tween(
            "alpha",
            0.5f32,
            1.0f32,
            Duration::from_secs(1),
            Ease::Linear,
            Position::Absolute(Duration::from_secs(1)), // [1s, 2s] - exactly abutting
        )
        .tween(
            "alpha",
            1.0f32,
            0.0f32,
            Duration::from_secs(1),
            Ease::Linear,
            Position::Absolute(Duration::from_secs(2)), // [2s, 3s] - exactly abutting
        );

    let compiled = builder.compile().expect("abutting intervals must succeed");
    let track = compiled.track::<f32>("alpha").unwrap();
    assert_eq!(track.clips().len(), 3);
    assert_eq!(track.clips()[0].end_time(), track.clips()[1].start_time);
    assert_eq!(track.clips()[1].end_time(), track.clips()[2].start_time);
}

#[test]
fn test_multi_track_concurrent_intervals_permitted() {
    let mut builder = TimelineBuilder::new();
    // Track 1 and Track 2 share the identical time interval [0s, 2s]
    builder
        .tween(
            "x",
            0.0f64,
            100.0f64,
            Duration::from_secs(2),
            Ease::QuadOut,
            Position::Absolute(Duration::ZERO),
        )
        .tween(
            "y",
            0.0f64,
            200.0f64,
            Duration::from_secs(2),
            Ease::QuadOut,
            Position::Absolute(Duration::ZERO),
        );

    let compiled = builder.compile().expect("parallel tracks must succeed");
    assert_eq!(compiled.duration(), Duration::from_secs(2));
    assert!(compiled.has_track("x"));
    assert!(compiled.has_track("y"));
}
