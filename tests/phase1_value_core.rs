use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_interpolation_numeric_and_arrays() {
    let scalar_start = 0.0f32;
    let scalar_end = 100.0f32;
    assert_eq!(scalar_start.interpolate(&scalar_end, 0.35), 35.0);

    let v3_start = [0.0f32, 10.0, 100.0];
    let v3_end = [10.0f32, 30.0, 200.0];
    let v3_mid = v3_start.interpolate(&v3_end, 0.5);
    assert_eq!(v3_mid, [5.0, 20.0, 150.0]);
}

#[test]
fn test_ease_endpoint_and_sampling() {
    let curves = [
        Ease::Linear,
        Ease::QuadIn,
        Ease::QuadOut,
        Ease::QuadInOut,
        Ease::CubicInOut,
        Ease::SineInOut,
        Ease::ExpoInOut,
        Ease::ElasticOut,
        Ease::BackOut,
        Ease::BounceOut,
    ];

    for curve in curves {
        assert!((curve.sample(0.0) - 0.0).abs() < 1e-4);
        assert!((curve.sample(1.0) - 1.0).abs() < 1e-4);
    }
}

#[test]
fn test_set_immediate_semantics() {
    let mut tween = Tween::set(999.0f64);

    assert!(tween.is_completed());
    assert_eq!(tween.value(), 999.0);
    assert_eq!(tween.progress(), 1.0);

    let (val, state) = tween.step(Duration::from_millis(100));
    assert_eq!(val, 999.0);
    assert_eq!(state, ClockState::Completed);
}

#[test]
fn test_from_to_sample_points() {
    let mut tween = Tween::from_to(0.0f32, 100.0f32, Duration::from_secs(10))
        .ease(Ease::Linear);

    let sample_points = [
        (Duration::from_secs(0), 0.0f32),
        (Duration::from_secs(2), 20.0f32),
        (Duration::from_secs(5), 50.0f32),
        (Duration::from_secs(8), 80.0f32),
        (Duration::from_secs(10), 100.0f32),
    ];

    for (time, expected) in sample_points {
        assert_eq!(tween.sample_at(time), expected);
    }

    // Step incrementally
    let (v, s) = tween.step(Duration::from_secs(5));
    assert_eq!(v, 50.0);
    assert_eq!(s, ClockState::Active);

    let (v, s) = tween.step(Duration::from_secs(5));
    assert_eq!(v, 100.0);
    assert_eq!(s, ClockState::Completed);
    assert!(tween.is_completed());
}

#[test]
fn test_repeat_correctness() {
    let mut tween = Tween::from_to(0.0f32, 100.0f32, Duration::from_secs(2))
        .ease(Ease::Linear)
        .repeat(3); // 3 cycles = 6 secs total

    assert_eq!(tween.clock().total_duration(), Some(Duration::from_secs(6)));

    // Cycle 1 at 1s (50%)
    let (v, s) = tween.step(Duration::from_secs(1));
    assert_eq!(v, 50.0);
    assert_eq!(s, ClockState::Active);

    // Cycle 1 boundary at 2s: cycle 2 has started, value wraps to 0.0
    let (v, s) = tween.step(Duration::from_secs(1));
    assert_eq!(v, 0.0);
    assert_eq!(s, ClockState::Active);

    // Cycle 2 at 3s (50%)
    let (v, s) = tween.step(Duration::from_secs(1));
    assert_eq!(v, 50.0);
    assert_eq!(s, ClockState::Active);

    // Run to completion (remaining 3s)
    let (v, s) = tween.step(Duration::from_secs(3));
    assert_eq!(v, 100.0);
    assert_eq!(s, ClockState::Completed);
    assert!(tween.is_completed());
}

#[test]
fn test_mirrored_yoyo_correctness() {
    let mut tween = Tween::from_to(10.0f32, 20.0f32, Duration::from_secs(1))
        .ease(Ease::Linear)
        .repeat(2)
        .yoyo(true);

    // 0s: start at 10.0
    assert_eq!(tween.value(), 10.0);

    // 0.5s: 15.0
    let (v, _) = tween.step(Duration::from_millis(500));
    assert_eq!(v, 15.0);

    // 1.0s: reaches peak 20.0
    let (v, _) = tween.step(Duration::from_millis(500));
    assert_eq!(v, 20.0);

    // 1.5s: yoyo back toward start, at 15.0
    let (v, _) = tween.step(Duration::from_millis(500));
    assert_eq!(v, 15.0);

    // 2.0s: returns to start at 10.0 and completes
    let (v, s) = tween.step(Duration::from_millis(500));
    assert_eq!(v, 10.0);
    assert_eq!(s, ClockState::Completed);
    assert!(tween.is_completed());
}

#[test]
fn test_seek_and_reverse_behavior() {
    let mut tween = Tween::from_to(0.0f32, 50.0f32, Duration::from_secs(5))
        .ease(Ease::Linear);

    // Seek directly to 4s (80% = 40.0)
    let val = tween.seek(Duration::from_secs(4));
    assert_eq!(val, 40.0);
    assert_eq!(tween.progress(), 0.8);

    // Reverse direction and step 2s backward
    tween.reverse();
    let (val, state) = tween.step(Duration::from_secs(2));
    assert_eq!(val, 20.0);
    assert_eq!(state, ClockState::Active);

    // Step 2s more to reach beginning (0s)
    let (val, state) = tween.step(Duration::from_secs(2));
    assert_eq!(val, 0.0);
    assert_eq!(state, ClockState::Completed);
    assert!(tween.is_completed());
}
