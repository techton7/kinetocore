use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_standard_repeat_finite() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "val",
        0.0f64,
        100.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::Absolute(Duration::ZERO),
    );

    let mut timeline = Timeline::new(builder.compile().unwrap());
    timeline
        .transport_mut()
        .set_repeat(3, RepeatStrategy::Repeat);

    assert_eq!(
        timeline.transport().total_duration(),
        Some(Duration::from_secs(3))
    );

    // --- Cycle 0 [0s, 1s] ---
    assert_eq!(timeline.time(), Duration::ZERO);
    assert_eq!(timeline.sample::<f64>("val"), Some(0.0));

    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert_eq!(timeline.sample::<f64>("val"), Some(50.0));

    // --- Cycle 1 [1s, 2s] ---
    timeline.step(Duration::from_millis(750)); // total elapsed = 1.25s
    assert_eq!(timeline.transport().elapsed(), Duration::from_millis(1250));
    assert_eq!(timeline.time(), Duration::from_millis(250));
    assert_eq!(timeline.sample::<f64>("val"), Some(25.0));

    timeline.step(Duration::from_millis(500)); // total elapsed = 1.75s
    assert_eq!(timeline.time(), Duration::from_millis(750));
    assert_eq!(timeline.sample::<f64>("val"), Some(75.0));

    // --- Cycle 2 [2s, 3s] ---
    timeline.step(Duration::from_millis(750)); // total elapsed = 2.5s
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert_eq!(timeline.sample::<f64>("val"), Some(50.0));

    // Finish cycle 2 at 3.0s
    let state = timeline.step(Duration::from_millis(500));
    assert_eq!(state, ClockState::Completed);
    assert!(timeline.is_completed());
    assert_eq!(timeline.transport().elapsed(), Duration::from_secs(3));
    assert_eq!(timeline.time(), Duration::from_secs(1));
    assert_eq!(timeline.sample::<f64>("val"), Some(100.0));
}

#[test]
fn test_yoyo_mirrored_repeat_finite() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "val",
        0.0f64,
        100.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::Absolute(Duration::ZERO),
    );

    let mut timeline = Timeline::new(builder.compile().unwrap());
    timeline
        .transport_mut()
        .set_repeat(2, RepeatStrategy::MirroredRepeat);

    assert_eq!(
        timeline.transport().total_duration(),
        Some(Duration::from_secs(2))
    );

    // --- Cycle 0 [0s, 1s] Forward: 0 -> 100 ---
    assert_eq!(timeline.time(), Duration::ZERO);
    assert_eq!(timeline.sample::<f64>("val"), Some(0.0));

    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert_eq!(timeline.sample::<f64>("val"), Some(50.0));

    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_secs(1));
    assert_eq!(timeline.sample::<f64>("val"), Some(100.0));

    // --- Cycle 1 [1s, 2s] Mirrored / Reverse: 100 -> 0 ---
    timeline.step(Duration::from_millis(250)); // elapsed = 1.25s -> mirrored time = 0.75s
    assert_eq!(timeline.time(), Duration::from_millis(750));
    assert_eq!(timeline.sample::<f64>("val"), Some(75.0));

    timeline.step(Duration::from_millis(250)); // elapsed = 1.50s -> mirrored time = 0.50s
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert_eq!(timeline.sample::<f64>("val"), Some(50.0));

    timeline.step(Duration::from_millis(250)); // elapsed = 1.75s -> mirrored time = 0.25s
    assert_eq!(timeline.time(), Duration::from_millis(250));
    assert_eq!(timeline.sample::<f64>("val"), Some(25.0));

    // Complete cycle 1 at 2.0s -> mirrored time = 0.0s
    let state = timeline.step(Duration::from_millis(250));
    assert_eq!(state, ClockState::Completed);
    assert!(timeline.is_completed());
    assert_eq!(timeline.time(), Duration::ZERO);
    assert_eq!(timeline.sample::<f64>("val"), Some(0.0));
}

#[test]
fn test_yoyo_mirrored_odd_cycles() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "x",
        10.0f64,
        20.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::Absolute(Duration::ZERO),
    );

    let mut timeline = Timeline::new(builder.compile().unwrap());
    timeline
        .transport_mut()
        .set_repeat(3, RepeatStrategy::MirroredRepeat);

    assert_eq!(
        timeline.transport().total_duration(),
        Some(Duration::from_secs(3))
    );

    // Fast-forward to cycle 2 (last cycle, elapsed = 2.5s)
    timeline.step(Duration::from_millis(2500));
    // Cycle 2 is forward (even cycle index 2): mirrored time = 0.5s
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert_eq!(timeline.sample::<f64>("x"), Some(15.0));

    // Finish 3rd cycle at 3.0s -> forward terminal value (20.0)
    let state = timeline.step(Duration::from_millis(500));
    assert_eq!(state, ClockState::Completed);
    assert_eq!(timeline.time(), Duration::from_secs(1));
    assert_eq!(timeline.sample::<f64>("x"), Some(20.0));
}

#[test]
fn test_infinite_repeat_wrapping() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "progress",
        0.0f64,
        1.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::Absolute(Duration::ZERO),
    );

    let compiled = builder.compile().unwrap();
    let mut timeline = Timeline::new(compiled.clone());
    timeline
        .transport_mut()
        .set_repeat(RepeatCount::Infinite, RepeatStrategy::Repeat);

    assert_eq!(timeline.transport().total_duration(), None);

    // Step across 50 full cycles plus 0.4s
    timeline.step(Duration::from_millis(50400));
    assert!(!timeline.is_completed());
    assert_eq!(timeline.transport().elapsed(), Duration::from_millis(50400));
    assert_eq!(timeline.time(), Duration::from_millis(400));
    let sample: f64 = timeline.sample("progress").unwrap();
    assert!((sample - 0.4).abs() < 1e-4);

    // Test infinite mirrored repeat
    let mut tl_yoyo = Timeline::new(compiled);
    tl_yoyo
        .transport_mut()
        .set_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);

    // Step 51.3s: 51 is odd cycle index, so mirrored time is 1.0 - 0.3 = 0.7s
    tl_yoyo.step(Duration::from_millis(51300));
    assert!(!tl_yoyo.is_completed());
    assert_eq!(tl_yoyo.time(), Duration::from_millis(700));
    let yoyo_sample: f64 = tl_yoyo.sample("progress").unwrap();
    assert!((yoyo_sample - 0.7).abs() < 1e-4);
}
