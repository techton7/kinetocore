use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_boundary_policy_f64_explicit_initial() {
    let mut builder = TimelineBuilder::new();
    builder.track_with_initial::<f64>("alpha", -10.0, |t| {
        t.from_to(
            0.0,
            10.0,
            Duration::from_secs(1),
            Ease::Linear,
            Position::Absolute(Duration::from_secs(1)), // [1s, 2s]
        )
        .from_to(
            20.0,
            50.0,
            Duration::from_millis(1500),
            Ease::Linear,
            Position::Absolute(Duration::from_millis(3500)), // [3.5s, 5s]
        );
    });

    let compiled = builder.compile().expect("compilation should succeed");
    let track = compiled.track::<f64>("alpha").expect("track alpha exists");

    assert_eq!(track.initial_value(), &-10.0);
    assert_eq!(compiled.duration(), Duration::from_millis(5000));

    // 1. Pre-roll (t < 1.0s): holds initial value -10.0
    assert_eq!(track.boundary_at(Duration::ZERO), Some(BoundaryPolicy::HoldInitial));
    assert_eq!(track.sample_at(Duration::ZERO), -10.0);
    assert_eq!(
        track.boundary_at(Duration::from_millis(500)),
        Some(BoundaryPolicy::HoldInitial)
    );
    assert_eq!(track.sample_at(Duration::from_millis(500)), -10.0);
    assert_eq!(
        track.boundary_at(Duration::from_millis(999)),
        Some(BoundaryPolicy::HoldInitial)
    );
    assert_eq!(track.sample_at(Duration::from_millis(999)), -10.0);

    // 2. Active Clip 1 ([1.0s, 2.0s])
    assert_eq!(track.boundary_at(Duration::from_secs(1)), None);
    assert_eq!(track.sample_at(Duration::from_secs(1)), 0.0);

    assert_eq!(track.boundary_at(Duration::from_millis(1500)), None);
    assert_eq!(track.sample_at(Duration::from_millis(1500)), 5.0);

    assert_eq!(track.sample_at(Duration::from_secs(2)), 10.0);

    // 3. Inter-Clip Gap ((2.0s, 3.5s)): forward-fill holds terminal value of Clip 1 (10.0)
    assert_eq!(
        track.boundary_at(Duration::from_millis(2001)),
        Some(BoundaryPolicy::ForwardFill)
    );
    assert_eq!(track.sample_at(Duration::from_millis(2001)), 10.0);

    assert_eq!(
        track.boundary_at(Duration::from_millis(2500)),
        Some(BoundaryPolicy::ForwardFill)
    );
    assert_eq!(track.sample_at(Duration::from_millis(2500)), 10.0);

    assert_eq!(
        track.boundary_at(Duration::from_millis(3499)),
        Some(BoundaryPolicy::ForwardFill)
    );
    assert_eq!(track.sample_at(Duration::from_millis(3499)), 10.0);

    // 4. Active Clip 2 ([3.5s, 5.0s])
    assert_eq!(track.boundary_at(Duration::from_millis(3500)), None);
    assert_eq!(track.sample_at(Duration::from_millis(3500)), 20.0);

    assert_eq!(track.boundary_at(Duration::from_millis(4250)), None);
    assert_eq!(track.sample_at(Duration::from_millis(4250)), 35.0);

    assert_eq!(track.sample_at(Duration::from_secs(5)), 50.0);

    // 5. Post-roll (t > 5.0s): holds terminal value of last clip (50.0)
    assert_eq!(
        track.boundary_at(Duration::from_millis(5001)),
        Some(BoundaryPolicy::HoldTerminal)
    );
    assert_eq!(track.sample_at(Duration::from_millis(5001)), 50.0);

    assert_eq!(
        track.boundary_at(Duration::from_secs(10)),
        Some(BoundaryPolicy::HoldTerminal)
    );
    assert_eq!(track.sample_at(Duration::from_secs(10)), 50.0);
}

#[test]
fn test_boundary_policy_f64_inferred_initial() {
    let mut builder = TimelineBuilder::new();
    builder.track::<f64>("pos", |t| {
        t.from_to(
            42.0,
            100.0,
            Duration::from_secs(1),
            Ease::Linear,
            Position::Absolute(Duration::from_millis(500)),
        );
    });

    let compiled = builder.compile().expect("compilation should succeed");
    let track = compiled.track::<f64>("pos").unwrap();

    // When not explicitly specified, initial_value defaults to first clip's start value (42.0)
    assert_eq!(track.initial_value(), &42.0);

    // Pre-roll (t < 0.5s)
    assert_eq!(track.boundary_at(Duration::ZERO), Some(BoundaryPolicy::HoldInitial));
    assert_eq!(track.sample_at(Duration::ZERO), 42.0);
    assert_eq!(track.sample_at(Duration::from_millis(250)), 42.0);

    // Active ([0.5s, 1.5s])
    assert_eq!(track.sample_at(Duration::from_millis(1000)), 71.0);

    // Post-roll (t > 1.5s)
    assert_eq!(
        track.boundary_at(Duration::from_secs(2)),
        Some(BoundaryPolicy::HoldTerminal)
    );
    assert_eq!(track.sample_at(Duration::from_secs(2)), 100.0);
}

#[test]
fn test_boundary_policy_array_vector_track() {
    let mut builder = TimelineBuilder::new();
    builder.track_with_initial::<[f32; 2]>("point", [1.0, 2.0], |t| {
        t.from_to(
            [10.0, 20.0],
            [30.0, 40.0],
            Duration::from_secs(1),
            Ease::Linear,
            Position::Absolute(Duration::from_secs(1)), // [1s, 2s]
        )
        .hold(
            [50.0, 60.0],
            Duration::from_secs(1),
            Position::Absolute(Duration::from_secs(4)), // [4s, 5s]
        );
    });

    let timeline = Timeline::new(builder.compile().expect("compilation should succeed"));
    let track = timeline.compiled().track::<[f32; 2]>("point").unwrap();

    // Pre-roll (t < 1s): returns [1.0, 2.0]
    assert_eq!(track.boundary_at(Duration::ZERO), Some(BoundaryPolicy::HoldInitial));
    assert_eq!(timeline.sample_at::<[f32; 2]>("point", Duration::ZERO), Some([1.0, 2.0]));
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("point", Duration::from_millis(500)),
        Some([1.0, 2.0])
    );

    // Active Tween ([1s, 2s])
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("point", Duration::from_millis(1500)),
        Some([20.0, 30.0])
    );
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("point", Duration::from_secs(2)),
        Some([30.0, 40.0])
    );

    // Gap ((2s, 4s)): forward-fills [30.0, 40.0]
    assert_eq!(
        track.boundary_at(Duration::from_secs(3)),
        Some(BoundaryPolicy::ForwardFill)
    );
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("point", Duration::from_secs(3)),
        Some([30.0, 40.0])
    );

    // Active Hold ([4s, 5s]): returns [50.0, 60.0]
    assert_eq!(track.boundary_at(Duration::from_millis(4500)), None);
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("point", Duration::from_millis(4500)),
        Some([50.0, 60.0])
    );

    // Post-roll (t > 5s): holds [50.0, 60.0]
    assert_eq!(
        track.boundary_at(Duration::from_secs(6)),
        Some(BoundaryPolicy::HoldTerminal)
    );
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("point", Duration::from_secs(6)),
        Some([50.0, 60.0])
    );
}

#[test]
fn test_concurrent_heterogeneous_tracks_boundary_sampling() {
    let mut builder = TimelineBuilder::new();
    builder
        .track_with_initial::<f64>("scalar", 0.0, |t| {
            t.from_to(
                100.0,
                200.0,
                Duration::from_secs(2),
                Ease::Linear,
                Position::Absolute(Duration::from_secs(1)), // [1s, 3s]
            );
        })
        .track_with_initial::<[f32; 2]>("coords", [0.0, 0.0], |t| {
            t.keyframes(
                vec![
                    Keyframe::linear(0.0, [10.0, 20.0]),
                    Keyframe::linear(0.5, [50.0, 100.0]),
                    Keyframe::linear(1.0, [100.0, 200.0]),
                ],
                Duration::from_secs(2),
                Position::Absolute(Duration::from_secs(2)), // [2s, 4s]
            );
        });

    let timeline = Timeline::new(builder.compile().expect("compilation should succeed"));
    assert_eq!(timeline.duration(), Duration::from_secs(4));

    // At t = 0.5s:
    // "scalar" is in pre-roll -> 0.0
    // "coords" is in pre-roll -> [0.0, 0.0]
    assert_eq!(timeline.sample_at::<f64>("scalar", Duration::from_millis(500)), Some(0.0));
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("coords", Duration::from_millis(500)),
        Some([0.0, 0.0])
    );

    // At t = 2.0s:
    // "scalar" is at t = 2s in [1s, 3s] -> 150.0
    // "coords" is at start of keyframes [2s, 4s] -> [10.0, 20.0]
    assert_eq!(timeline.sample_at::<f64>("scalar", Duration::from_secs(2)), Some(150.0));
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("coords", Duration::from_secs(2)),
        Some([10.0, 20.0])
    );

    // At t = 3.0s:
    // "scalar" is at end of [1s, 3s] -> 200.0
    // "coords" is at midpoint of keyframes (offset 0.5) -> [50.0, 100.0]
    assert_eq!(timeline.sample_at::<f64>("scalar", Duration::from_secs(3)), Some(200.0));
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("coords", Duration::from_secs(3)),
        Some([50.0, 100.0])
    );

    // At t = 3.5s:
    // "scalar" is in post-roll (> 3s) -> holds 200.0
    // "coords" is at 3/4 progress -> [75.0, 150.0]
    assert_eq!(timeline.sample_at::<f64>("scalar", Duration::from_millis(3500)), Some(200.0));
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("coords", Duration::from_millis(3500)),
        Some([75.0, 150.0])
    );

    // At t = 5.0s:
    // Both are in post-roll
    assert_eq!(timeline.sample_at::<f64>("scalar", Duration::from_secs(5)), Some(200.0));
    assert_eq!(
        timeline.sample_at::<[f32; 2]>("coords", Duration::from_secs(5)),
        Some([100.0, 200.0])
    );
}
