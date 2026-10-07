use std::time::Duration;

use kinetocore::prelude::*;

const EPSILON: f64 = 1e-10;

#[test]
fn test_reverse_tween_evaluation_symmetry() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "progress",
        0.0f64,
        100.0f64,
        Duration::from_secs(5),
        Ease::CubicInOut,
        Position::Absolute(Duration::ZERO),
    );

    let compiled = builder.compile().expect("compilation should succeed");
    let mut timeline = Timeline::new(compiled);

    // Play forward to completion
    let step_dt = Duration::from_millis(50);
    let total_steps = 100; // 5 seconds / 50ms

    let mut forward_values = Vec::with_capacity(total_steps + 1);
    forward_values.push((timeline.time(), timeline.sample::<f64>("progress").unwrap()));

    for _ in 0..total_steps {
        timeline.step(step_dt);
        forward_values.push((timeline.time(), timeline.sample::<f64>("progress").unwrap()));
    }

    assert!(timeline.is_completed());
    assert_eq!(timeline.time(), Duration::from_secs(5));
    assert!((timeline.sample::<f64>("progress").unwrap() - 100.0).abs() < EPSILON);

    // Reverse direction and resume playing backward
    timeline.reverse();
    timeline.play();
    assert_eq!(
        timeline.transport().direction(),
        PlaybackDirection::Backward
    );
    assert!(!timeline.is_completed());

    // Step backward and verify exact symmetry
    for expected in forward_values.into_iter().rev().skip(1) {
        let state = timeline.step(step_dt);
        let actual_val: f64 = timeline.sample("progress").unwrap();
        assert_eq!(timeline.time(), expected.0);
        assert!(
            (actual_val - expected.1).abs() < EPSILON,
            "Reverse sample mismatch at time {:?}: expected {}, got {}",
            timeline.time(),
            expected.1,
            actual_val
        );

        if timeline.time() == Duration::ZERO {
            assert_eq!(state, ClockState::Completed);
        } else {
            assert_eq!(state, ClockState::Active);
        }
    }

    assert!(timeline.is_completed());
    assert_eq!(timeline.time(), Duration::ZERO);
    assert!((timeline.sample::<f64>("progress").unwrap() - 0.0).abs() < EPSILON);
}

#[test]
fn test_reverse_keyframes_multi_segment_symmetry() {
    let mut builder = TimelineBuilder::new();
    builder.keyframes(
        "trajectory",
        vec![
            Keyframe::linear(0.0, 10.0f64),
            Keyframe::new(0.2, 50.0f64, Ease::QuadIn),
            Keyframe::new(0.6, 20.0f64, Ease::CubicOut),
            Keyframe::new(0.9, 100.0f64, Ease::SineInOut),
            Keyframe::linear(1.0, 80.0f64),
        ],
        Duration::from_secs(10),
        Position::Absolute(Duration::ZERO),
    );

    let compiled = builder.compile().expect("compilation should succeed");
    let mut timeline = Timeline::new(compiled);

    // Collect forward trajectory at 100ms intervals
    let dt = Duration::from_millis(100);
    let mut trajectory = Vec::new();
    while !timeline.is_completed() {
        trajectory.push((timeline.time(), timeline.sample::<f64>("trajectory").unwrap()));
        timeline.step(dt);
    }
    trajectory.push((timeline.time(), timeline.sample::<f64>("trajectory").unwrap()));

    assert_eq!(timeline.time(), Duration::from_secs(10));
    assert!((timeline.sample::<f64>("trajectory").unwrap() - 80.0).abs() < EPSILON);

    // Reverse and play backward
    timeline.reverse();
    timeline.play();
    for expected in trajectory.into_iter().rev().skip(1) {
        timeline.step(dt);
        let actual: f64 = timeline.sample("trajectory").unwrap();
        assert_eq!(timeline.time(), expected.0);
        assert!(
            (actual - expected.1).abs() < EPSILON,
            "Keyframe reverse mismatch at {:?}: expected {}, got {}",
            timeline.time(),
            expected.1,
            actual
        );
    }

    assert!(timeline.is_completed());
    assert_eq!(timeline.time(), Duration::ZERO);
    assert!((timeline.sample::<f64>("trajectory").unwrap() - 10.0).abs() < EPSILON);
}

#[test]
fn test_reverse_interspersed_sets_and_tweens() {
    let mut builder = TimelineBuilder::new();
    // 0s..2s: Tween from 0 to 50
    // 2s: Set to 200
    // 3s..5s: Tween from 200 to 300
    builder
        .tween(
            "val",
            0.0f64,
            50.0f64,
            Duration::from_secs(2),
            Ease::Linear,
            Position::Absolute(Duration::ZERO),
        )
        .set("val", 200.0f64, Position::Absolute(Duration::from_secs(2)))
        .tween(
            "val",
            200.0f64,
            300.0f64,
            Duration::from_secs(2),
            Ease::Linear,
            Position::Absolute(Duration::from_secs(3)),
        );

    let compiled = builder.compile().expect("compilation should succeed");
    let mut timeline = Timeline::new(compiled);

    // Seek to end
    timeline.seek(Duration::from_secs(5));
    assert!((timeline.sample::<f64>("val").unwrap() - 300.0).abs() < EPSILON);

    // Reverse
    timeline.reverse();

    // Step backward into middle of second tween (4s -> halfway between 200 and 300 = 250)
    timeline.step(Duration::from_secs(1));
    assert_eq!(timeline.time(), Duration::from_secs(4));
    assert!((timeline.sample::<f64>("val").unwrap() - 250.0).abs() < EPSILON);

    // Step backward to start of second tween (3s -> 200)
    timeline.step(Duration::from_secs(1));
    assert_eq!(timeline.time(), Duration::from_secs(3));
    assert!((timeline.sample::<f64>("val").unwrap() - 200.0).abs() < EPSILON);

    // Step backward into gap (2.5s -> forward-fill from set at 2s = 200)
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_millis(2500));
    assert!((timeline.sample::<f64>("val").unwrap() - 200.0).abs() < EPSILON);

    // Step backward to 2s: Set clip terminal is 200
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_secs(2));
    assert!((timeline.sample::<f64>("val").unwrap() - 200.0).abs() < EPSILON);

    // Step backward into first tween (1s -> halfway between 0 and 50 = 25)
    timeline.step(Duration::from_secs(1));
    assert_eq!(timeline.time(), Duration::from_secs(1));
    assert!((timeline.sample::<f64>("val").unwrap() - 25.0).abs() < EPSILON);

    // Step backward to 0s -> 0
    timeline.step(Duration::from_secs(1));
    assert_eq!(timeline.time(), Duration::ZERO);
    assert!((timeline.sample::<f64>("val").unwrap() - 0.0).abs() < EPSILON);
    assert!(timeline.is_completed());
}

#[test]
fn test_standalone_tween_reverse_clock_algebra() {
    let mut tween =
        Tween::from_to(0.0f64, 100.0f64, Duration::from_secs(4)).ease(Ease::QuadInOut);

    // Step forward 2 seconds (halfway)
    let (val, state) = tween.step(Duration::from_secs(2));
    assert_eq!(state, ClockState::Active);
    assert!((val - 50.0).abs() < EPSILON);
    assert_eq!(tween.progress(), 0.5);

    // Reverse direction
    tween.reverse();
    assert_eq!(tween.clock().direction(), PlaybackDirection::Backward);

    // Step backward 1 second (elapsed = 1s, QuadInOut at 0.25 -> 2 * 0.25^2 = 0.125 -> 12.5)
    let (val, state) = tween.step(Duration::from_secs(1));
    assert_eq!(state, ClockState::Active);
    assert_eq!(tween.clock().elapsed(), Duration::from_secs(1));
    assert!((val - 12.5).abs() < EPSILON);

    // Step backward 1 second (elapsed = 0s -> completed)
    let (val, state) = tween.step(Duration::from_secs(1));
    assert_eq!(state, ClockState::Completed);
    assert_eq!(tween.clock().elapsed(), Duration::ZERO);
    assert!((val - 0.0).abs() < EPSILON);
    assert!(tween.is_completed());
}

#[test]
fn test_reverse_multi_track_boundary_policies() {
    let mut builder = TimelineBuilder::new();
    // Track 1: f64 active from 1s to 3s
    builder.tween(
        "x",
        10.0f64,
        50.0f64,
        Duration::from_secs(2),
        Ease::Linear,
        Position::Absolute(Duration::from_secs(1)),
    );
    // Track 2: [f32; 2] active from 2s to 4s
    builder.tween(
        "pos",
        [0.0f32, 0.0f32],
        [100.0f32, 200.0f32],
        Duration::from_secs(2),
        Ease::Linear,
        Position::Absolute(Duration::from_secs(2)),
    );

    let compiled = builder.compile().expect("compilation should succeed");
    let mut timeline = Timeline::new(compiled.clone());

    assert_eq!(compiled.duration(), Duration::from_secs(4));

    // Seek to 4s (terminal boundary)
    timeline.seek(Duration::from_secs(4));
    assert_eq!(timeline.time(), Duration::from_secs(4));
    assert_eq!(timeline.sample::<f64>("x").unwrap(), 50.0);
    assert_eq!(timeline.sample::<[f32; 2]>("pos").unwrap(), [100.0, 200.0]);

    // Reverse and step backwards
    timeline.reverse();
    timeline.play();

    // 3.5s: Track x is in post-roll (terminal 50.0), Track pos is active halfway ([75.0, 150.0])
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_millis(3500));
    assert_eq!(timeline.sample::<f64>("x").unwrap(), 50.0);
    let p35 = timeline.sample::<[f32; 2]>("pos").unwrap();
    assert!((p35[0] - 75.0).abs() < 1e-4);
    assert!((p35[1] - 150.0).abs() < 1e-4);

    // 2.0s: Track x is at 30.0 (halfway), Track pos is at start [0.0, 0.0]
    timeline.step(Duration::from_millis(1500));
    assert_eq!(timeline.time(), Duration::from_secs(2));
    assert!((timeline.sample::<f64>("x").unwrap() - 30.0).abs() < 1e-4);
    assert_eq!(timeline.sample::<[f32; 2]>("pos").unwrap(), [0.0, 0.0]);

    // 0.5s: Track x is in pre-roll (initial 10.0), Track pos is in pre-roll ([0.0, 0.0])
    timeline.step(Duration::from_millis(1500));
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert_eq!(timeline.sample::<f64>("x").unwrap(), 10.0);
    assert_eq!(timeline.sample::<[f32; 2]>("pos").unwrap(), [0.0, 0.0]);

    // 0.0s: Boundary
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::ZERO);
    assert_eq!(timeline.sample::<f64>("x").unwrap(), 10.0);
    assert_eq!(timeline.sample::<[f32; 2]>("pos").unwrap(), [0.0, 0.0]);
    assert!(timeline.is_completed());
}

#[test]
fn test_reverse_yoyo_repeat_schedules() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "val",
        0.0f64,
        100.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::Absolute(Duration::ZERO),
    );

    let compiled = builder.compile().unwrap();
    let mut timeline = Timeline::new(compiled);

    // Set 2 cycles with mirrored repeat (cycle 1: 0->100, cycle 2: 100->0)
    timeline
        .transport_mut()
        .set_repeat(2, RepeatStrategy::MirroredRepeat);

    // Cycle 1 halfway (0.5s): val = 50.0
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert!((timeline.sample::<f64>("val").unwrap() - 50.0).abs() < 1e-4);

    // Cycle 1 complete (1.0s): val = 100.0
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_secs(1));
    assert!((timeline.sample::<f64>("val").unwrap() - 100.0).abs() < 1e-4);

    // Cycle 2 halfway (1.5s): mirrored time = 0.5s, val = 50.0
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert!((timeline.sample::<f64>("val").unwrap() - 50.0).abs() < 1e-4);

    // Cycle 2 complete (2.0s): total completed, mirrored time = 0.0s, val = 0.0
    let state = timeline.step(Duration::from_millis(500));
    assert_eq!(state, ClockState::Completed);
    assert_eq!(timeline.time(), Duration::ZERO);
    assert!((timeline.sample::<f64>("val").unwrap() - 0.0).abs() < 1e-4);
    assert!(timeline.is_completed());

    // Reverse and step back
    timeline.reverse();
    timeline.play();

    // Step backward 500ms: elapsed goes from 2.0s to 1.5s -> mirrored time = 0.5s, val = 50.0
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_millis(500));
    assert!((timeline.sample::<f64>("val").unwrap() - 50.0).abs() < 1e-4);

    // Step backward another 500ms: elapsed goes from 1.5s to 1.0s -> cycle 1 end, val = 100.0
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.time(), Duration::from_secs(1));
    assert!((timeline.sample::<f64>("val").unwrap() - 100.0).abs() < 1e-4);

    // Step backward 1000ms: elapsed goes to 0 -> completion in reverse
    let state = timeline.step(Duration::from_secs(1));
    assert_eq!(state, ClockState::Completed);
    assert_eq!(timeline.time(), Duration::ZERO);
    assert!((timeline.sample::<f64>("val").unwrap() - 0.0).abs() < 1e-4);
    assert!(timeline.is_completed());
}


