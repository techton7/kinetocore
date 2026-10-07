use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_heterogeneous_multi_track_compilation() {
    let mut builder = TimelineBuilder::new();

    builder
        .track::<f64>("translation_x", |t| {
            t.set(0.0, Position::Absolute(Duration::ZERO))
                .to(
                    0.0,
                    150.0,
                    Duration::from_secs(1),
                    Ease::QuadOut,
                    Position::RecentEnd,
                )
                .hold(150.0, Duration::from_millis(500), Position::RecentEnd);
        })
        .add_label("x_moved", Position::RecentEnd)
        .track::<f32>("opacity", |t| {
            t.set(0.0, Position::Absolute(Duration::ZERO))
                .to(
                    0.0,
                    1.0,
                    Duration::from_millis(800),
                    Ease::CubicInOut,
                    Position::Label("x_moved".to_string()),
                );
        })
        .track::<[f32; 3]>("rgb", |t| {
            t.set([1.0, 0.0, 0.0], Position::Absolute(Duration::ZERO))
                .to(
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    Duration::from_secs(2),
                    Ease::Linear,
                    Position::LabelOffset(
                        "x_moved".to_string(),
                        SignedDuration::positive(Duration::from_millis(200)),
                    ),
                );
        });

    let compiled = builder.compile().expect("compilation should succeed");

    assert_eq!(compiled.track_count(), 3);
    assert_eq!(
        compiled.label("x_moved"),
        Some(Duration::from_millis(1500))
    );

    // translation_x: [0s, 0s], [0s, 1s], [1s, 1.5s] -> track dur = 1.5s
    let tx = compiled.track::<f64>("translation_x").unwrap();
    assert_eq!(tx.duration(), Duration::from_millis(1500));
    assert_eq!(tx.clip_count(), 3);

    // opacity: [0s, 0s], [1.5s, 2.3s] -> track dur = 2.3s
    let op = compiled.track::<f32>("opacity").unwrap();
    assert_eq!(op.duration(), Duration::from_millis(2300));
    assert_eq!(op.clip_count(), 2);

    // rgb: [0s, 0s], [1.7s, 3.7s] -> track dur = 3.7s
    let rgb = compiled.track::<[f32; 3]>("rgb").unwrap();
    assert_eq!(rgb.duration(), Duration::from_millis(3700));
    assert_eq!(rgb.clip_count(), 2);

    // Total duration is max across all tracks = 3.7s
    assert_eq!(compiled.duration(), Duration::from_millis(3700));
}
