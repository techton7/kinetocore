use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_keyframe_validation_rules() {
    // Exactly 2 keyframes at 0.0 and 1.0 is valid
    let min_kfs = vec![
        Keyframe::linear(0.0, 0.0f32),
        Keyframe::linear(1.0, 100.0f32),
    ];
    assert!(validate_keyframes(&min_kfs).is_ok());

    // Multi-keyframe sequence strictly increasing
    let multi_kfs = vec![
        Keyframe::linear(0.0, 0.0f32),
        Keyframe::new(0.2, 20.0f32, Ease::QuadIn),
        Keyframe::new(0.6, 80.0f32, Ease::QuadOut),
        Keyframe::linear(1.0, 100.0f32),
    ];
    assert!(validate_keyframes(&multi_kfs).is_ok());

    // Missing initial 0.0
    let err_start = vec![
        Keyframe::linear(0.05, 0.0f32),
        Keyframe::linear(1.0, 100.0f32),
    ];
    assert!(matches!(
        validate_keyframes(&err_start),
        Err(TimelineError::InvalidKeyframe(_))
    ));

    // Missing final 1.0
    let err_end = vec![
        Keyframe::linear(0.0, 0.0f32),
        Keyframe::linear(0.95, 100.0f32),
    ];
    assert!(matches!(
        validate_keyframes(&err_end),
        Err(TimelineError::InvalidKeyframe(_))
    ));

    // Non-increasing duplicate
    let err_dup = vec![
        Keyframe::linear(0.0, 0.0f32),
        Keyframe::linear(0.5, 50.0f32),
        Keyframe::linear(0.5, 60.0f32),
        Keyframe::linear(1.0, 100.0f32),
    ];
    assert!(matches!(
        validate_keyframes(&err_dup),
        Err(TimelineError::InvalidKeyframe(_))
    ));

    // Inverted ordering
    let err_inv = vec![
        Keyframe::linear(0.0, 0.0f32),
        Keyframe::linear(0.7, 70.0f32),
        Keyframe::linear(0.3, 30.0f32),
        Keyframe::linear(1.0, 100.0f32),
    ];
    assert!(matches!(
        validate_keyframes(&err_inv),
        Err(TimelineError::InvalidKeyframe(_))
    ));
}

#[test]
fn test_keyframe_builder_compilation_integration() {
    let mut builder = TimelineBuilder::new();
    let kfs = vec![
        Keyframe::linear(0.0, 0.0f64),
        Keyframe::new(0.3, 50.0f64, Ease::CubicInOut),
        Keyframe::new(0.7, 20.0f64, Ease::QuadOut),
        Keyframe::linear(1.0, 100.0f64),
    ];

    builder.keyframes(
        "kf_track",
        kfs,
        Duration::from_secs(2),
        Position::Absolute(Duration::from_millis(500)),
    );

    let compiled = builder.compile().expect("compilation should succeed");
    let track = compiled.track::<f64>("kf_track").unwrap();
    assert_eq!(track.clips().len(), 1);
    assert_eq!(track.clips()[0].start_time, Duration::from_millis(500));
    assert_eq!(track.clips()[0].duration, Duration::from_secs(2));
    assert_eq!(track.clips()[0].end_time(), Duration::from_millis(2500));
}
