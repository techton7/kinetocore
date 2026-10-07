use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_instant_clip_coexistence_and_properties() {
    let mut builder = TimelineBuilder::new();
    builder
        .set("opacity", 0.0f32, Position::Absolute(Duration::ZERO))
        .tween(
            "opacity",
            0.0f32,
            1.0f32,
            Duration::from_secs(1),
            Ease::Linear,
            Position::Absolute(Duration::ZERO), // Coexists at t = 0 with Set clip
        )
        .set(
            "opacity",
            0.5f32,
            Position::Absolute(Duration::from_secs(1)), // Set at end boundary
        );

    let compiled = builder.compile().expect("compilation should succeed");
    let track = compiled.track::<f32>("opacity").unwrap();
    assert_eq!(track.clips().len(), 3);

    assert!(track.clips()[0].is_instant());
    assert_eq!(track.clips()[0].duration, Duration::ZERO);
    assert_eq!(track.clips()[0].start_time, Duration::ZERO);

    assert!(track.clips()[1].is_spanned());
    assert_eq!(track.clips()[1].duration, Duration::from_secs(1));
    assert_eq!(track.clips()[1].start_time, Duration::ZERO);

    assert!(track.clips()[2].is_instant());
    assert_eq!(track.clips()[2].duration, Duration::ZERO);
    assert_eq!(track.clips()[2].start_time, Duration::from_secs(1));
}
