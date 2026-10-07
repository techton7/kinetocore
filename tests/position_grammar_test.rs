use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_position_grammar_all_string_variants() {
    // Empty -> End
    assert_eq!(Position::parse("").unwrap(), Position::End);
    assert_eq!(Position::parse("   ").unwrap(), Position::End);

    // Absolute
    assert_eq!(
        Position::parse("2.5").unwrap(),
        Position::Absolute(Duration::from_secs_f64(2.5))
    );
    assert_eq!(
        Position::parse("3s").unwrap(),
        Position::Absolute(Duration::from_secs(3))
    );
    assert_eq!(
        Position::parse("750ms").unwrap(),
        Position::Absolute(Duration::from_millis(750))
    );

    // Relative offsets
    assert_eq!(
        Position::parse("+=0.5s").unwrap(),
        Position::Offset(SignedDuration::positive(Duration::from_millis(500)))
    );
    assert_eq!(
        Position::parse("+=300ms").unwrap(),
        Position::Offset(SignedDuration::positive(Duration::from_millis(300)))
    );
    assert_eq!(
        Position::parse("-=0.25s").unwrap(),
        Position::Offset(SignedDuration::negative(Duration::from_millis(250)))
    );
    assert_eq!(
        Position::parse("-=150ms").unwrap(),
        Position::Offset(SignedDuration::negative(Duration::from_millis(150)))
    );

    // Recent anchors
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

    // Labels
    assert_eq!(
        Position::parse("header_enter").unwrap(),
        Position::Label("header_enter".to_string())
    );
    assert_eq!(
        Position::parse("header_enter+=0.4s").unwrap(),
        Position::LabelOffset(
            "header_enter".to_string(),
            SignedDuration::positive(Duration::from_millis(400))
        )
    );
    assert_eq!(
        Position::parse("header_enter-=0.1s").unwrap(),
        Position::LabelOffset(
            "header_enter".to_string(),
            SignedDuration::negative(Duration::from_millis(100))
        )
    );
}

#[test]
fn test_position_conversions_and_try_from() {
    let p_str: Position = "1.5s".try_into().unwrap();
    assert_eq!(p_str, Position::Absolute(Duration::from_secs_f64(1.5)));

    let p_string: Position = String::from("+=200ms").try_into().unwrap();
    assert_eq!(
        p_string,
        Position::Offset(SignedDuration::positive(Duration::from_millis(200)))
    );

    let p_dur: Position = Duration::from_secs(4).into();
    assert_eq!(p_dur, Position::Absolute(Duration::from_secs(4)));

    let p_f64: Position = 2.5f64.into();
    assert_eq!(p_f64, Position::Absolute(Duration::from_secs_f64(2.5)));

    let p_f32: Position = 1.25f32.into();
    assert_eq!(p_f32, Position::Absolute(Duration::from_secs_f64(1.25)));
}

#[test]
fn test_sequential_position_resolution_in_timeline() {
    let mut builder = TimelineBuilder::new();

    // Clip 1: [0.0s, 1.0s]
    builder.tween(
        "t1",
        0.0f64,
        10.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::End,
    );

    // Clip 2: RecentStart -> [0.0s, 2.0s] on track t2
    builder.tween(
        "t2",
        0.0f64,
        20.0f64,
        Duration::from_secs(2),
        Ease::Linear,
        Position::RecentStart,
    );

    // Clip 3: RecentStartOffset(+0.5s) relative to clip 2 start (0.0s) -> [0.5s, 1.5s] on track t3
    builder.tween(
        "t3",
        0.0f64,
        30.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::RecentStartOffset(SignedDuration::positive(Duration::from_millis(500))),
    );

    // Clip 4: RecentEndOffset(-0.2s) relative to clip 3 end (1.5s) -> [1.3s, 2.3s] on track t4
    builder.tween(
        "t4",
        0.0f64,
        40.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::RecentEndOffset(SignedDuration::negative(Duration::from_millis(200))),
    );

    // Label: at RecentEnd of clip 4 (2.3s)
    builder.add_label("marker", Position::RecentEnd);

    // Clip 5: LabelOffset("marker", +0.7s) -> [3.0s, 4.0s] on track t1
    builder.tween(
        "t1",
        10.0f64,
        50.0f64,
        Duration::from_secs(1),
        Ease::Linear,
        Position::LabelOffset(
            "marker".to_string(),
            SignedDuration::positive(Duration::from_millis(700)),
        ),
    );

    let compiled = builder.compile().expect("compilation should succeed");

    assert_eq!(compiled.label("marker"), Some(Duration::from_millis(2300)));
    assert_eq!(compiled.duration(), Duration::from_secs(4));

    let track_t1 = compiled.track::<f64>("t1").unwrap();
    assert_eq!(track_t1.clips().len(), 2);
    assert_eq!(track_t1.clips()[0].start_time, Duration::ZERO);
    assert_eq!(track_t1.clips()[0].end_time(), Duration::from_secs(1));
    assert_eq!(track_t1.clips()[1].start_time, Duration::from_secs(3));
    assert_eq!(track_t1.clips()[1].end_time(), Duration::from_secs(4));

    let track_t2 = compiled.track::<f64>("t2").unwrap();
    assert_eq!(track_t2.clips()[0].start_time, Duration::ZERO);
    assert_eq!(track_t2.clips()[0].end_time(), Duration::from_secs(2));

    let track_t3 = compiled.track::<f64>("t3").unwrap();
    assert_eq!(track_t3.clips()[0].start_time, Duration::from_millis(500));
    assert_eq!(track_t3.clips()[0].end_time(), Duration::from_millis(1500));

    let track_t4 = compiled.track::<f64>("t4").unwrap();
    assert_eq!(track_t4.clips()[0].start_time, Duration::from_millis(1300));
    assert_eq!(track_t4.clips()[0].end_time(), Duration::from_millis(2300));
}
