use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_continuous_step_vs_random_seek_cadence_independence() {
    let mut builder = TimelineBuilder::new();
    builder
        .tween(
            "linear",
            0.0f64,
            1000.0f64,
            Duration::from_secs(10),
            Ease::Linear,
            Position::Absolute(Duration::ZERO),
        )
        .tween(
            "quad",
            10.0f64,
            50.0f64,
            Duration::from_secs(10),
            Ease::QuadInOut,
            Position::Absolute(Duration::ZERO),
        )
        .tween(
            "cubic",
            -100.0f64,
            100.0f64,
            Duration::from_secs(10),
            Ease::CubicOut,
            Position::Absolute(Duration::ZERO),
        )
        .keyframes(
            "keyframe",
            vec![
                Keyframe::linear(0.0, 0.0f64),
                Keyframe::new(0.25, 250.0f64, Ease::QuadIn),
                Keyframe::new(0.75, 750.0f64, Ease::QuadOut),
                Keyframe::linear(1.0, 1000.0f64),
            ],
            Duration::from_secs(10),
            Position::Absolute(Duration::ZERO),
        );

    let compiled = builder.compile().expect("compilation should succeed");
    let total_dur = compiled.duration();
    assert_eq!(total_dur, Duration::from_secs(10));

    // Test across 100 distinct checkpoints
    let checkpoints_count = 100;
    for i in 0..=checkpoints_count {
        let t_target = Duration::from_millis(i * 100); // 0.0s, 0.1s, 0.2s, ..., 10.0s

        // 1. Continuous step: accumulate time from 0 via irregular deltas
        let mut tl_step = Timeline::new(compiled.clone());
        let mut cur = Duration::ZERO;
        let mut step_idx = 0;
        let deltas = [
            Duration::from_millis(16),
            Duration::from_millis(33),
            Duration::from_millis(8),
            Duration::from_millis(25),
        ];
        while cur < t_target {
            let dt = deltas[step_idx % deltas.len()].min(t_target - cur);
            tl_step.step(dt);
            cur += dt;
            step_idx += 1;
        }
        assert_eq!(tl_step.time(), t_target);

        // 2. Direct random-access seek
        let mut tl_seek = Timeline::new(compiled.clone());
        // Jump arbitrarily first, then seek to target
        tl_seek.seek(Duration::from_secs(8));
        tl_seek.seek(Duration::from_secs(1));
        tl_seek.seek(t_target);
        assert_eq!(tl_seek.time(), t_target);

        // 3. Stateless sampling
        let stateless_linear: f64 = compiled.sample_track("linear", t_target).unwrap();
        let stateless_quad: f64 = compiled.sample_track("quad", t_target).unwrap();
        let stateless_cubic: f64 = compiled.sample_track("cubic", t_target).unwrap();
        let stateless_kfs: f64 = compiled.sample_track("keyframe", t_target).unwrap();

        // Sample stepped timeline
        let step_linear: f64 = tl_step.sample("linear").unwrap();
        let step_quad: f64 = tl_step.sample("quad").unwrap();
        let step_cubic: f64 = tl_step.sample("cubic").unwrap();
        let step_kfs: f64 = tl_step.sample("keyframe").unwrap();

        // Sample seek timeline
        let seek_linear: f64 = tl_seek.sample("linear").unwrap();
        let seek_quad: f64 = tl_seek.sample("quad").unwrap();
        let seek_cubic: f64 = tl_seek.sample("cubic").unwrap();
        let seek_kfs: f64 = tl_seek.sample("keyframe").unwrap();

        // Assert mathematical equivalence with Δ < 10^-12
        let eps = 1e-12;

        assert!(
            (step_linear - seek_linear).abs() < eps,
            "Linear mismatch at {:?}: step={}, seek={}",
            t_target,
            step_linear,
            seek_linear
        );
        assert!((step_linear - stateless_linear).abs() < eps);

        assert!(
            (step_quad - seek_quad).abs() < eps,
            "Quad mismatch at {:?}: step={}, seek={}",
            t_target,
            step_quad,
            seek_quad
        );
        assert!((step_quad - stateless_quad).abs() < eps);

        assert!(
            (step_cubic - seek_cubic).abs() < eps,
            "Cubic mismatch at {:?}: step={}, seek={}",
            t_target,
            step_cubic,
            seek_cubic
        );
        assert!((step_cubic - stateless_cubic).abs() < eps);

        assert!(
            (step_kfs - seek_kfs).abs() < eps,
            "Keyframes mismatch at {:?}: step={}, seek={}",
            t_target,
            step_kfs,
            seek_kfs
        );
        assert!((step_kfs - stateless_kfs).abs() < eps);
    }
}

#[test]
fn test_scrubber_random_sweeps() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "x",
        0.0f64,
        500.0f64,
        Duration::from_secs(5),
        Ease::QuadInOut,
        Position::Absolute(Duration::ZERO),
    );

    let compiled = builder.compile().unwrap();
    let mut timeline = Timeline::new(compiled.clone());

    // Pseudo-random scrub jump pattern simulating frantic user scrubber interaction
    let jumps = [
        Duration::from_millis(4200),
        Duration::from_millis(150),
        Duration::from_millis(3800),
        Duration::from_millis(900),
        Duration::from_millis(2500),
        Duration::from_millis(4999),
        Duration::from_millis(0),
        Duration::from_millis(5000),
        Duration::from_millis(2500),
    ];

    for &t in &jumps {
        timeline.seek(t);
        let sampled_x: f64 = timeline.sample("x").unwrap();
        let expected_x: f64 = compiled.sample_track("x", t).unwrap();
        assert!(
            (sampled_x - expected_x).abs() < 1e-12,
            "Scrubber seek failed at {:?}: got {}, expected {}",
            t,
            sampled_x,
            expected_x
        );
    }
}

#[test]
fn test_scrubber_progress_exact_roundtrip() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "y",
        0.0f64,
        200.0f64,
        Duration::from_secs(10),
        Ease::Linear,
        Position::Absolute(Duration::ZERO),
    );

    let mut timeline = Timeline::new(builder.compile().unwrap());

    let test_fractions = [0.0f32, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];

    for &p in &test_fractions {
        timeline.set_progress(p);
        assert!((timeline.progress() - p).abs() < 1e-6);

        let expected_time_nanos = (Duration::from_secs(10).as_nanos() as f64 * p as f64).round() as u64;
        assert_eq!(timeline.time(), Duration::from_nanos(expected_time_nanos));

        let expected_val = 200.0 * (p as f64);
        let actual_val: f64 = timeline.sample("y").unwrap();
        assert!(
            (actual_val - expected_val).abs() < 1e-4,
            "Progress evaluation mismatch at progress {}: actual={}, expected={}",
            p,
            actual_val,
            expected_val
        );
    }
}
