use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_transport_initial_state_and_stepping() {
    let mut transport = TimelineTransport::new(Duration::from_secs(2));

    assert!(transport.is_playing());
    assert!(!transport.is_paused());
    assert!(!transport.is_completed());
    assert_eq!(transport.state(), TransportState::Playing);
    assert_eq!(transport.time(), Duration::ZERO);
    assert_eq!(transport.progress(), 0.0);

    let state = transport.step(Duration::from_millis(500));
    assert_eq!(state, ClockState::Active);
    assert_eq!(transport.time(), Duration::from_millis(500));
    assert!((transport.progress() - 0.25).abs() < 1e-4);
    assert!(transport.is_playing());
    assert!(!transport.is_completed());
}

#[test]
fn test_transport_pause_and_resume() {
    let mut transport = TimelineTransport::new(Duration::from_secs(2));

    transport.pause();
    assert!(transport.is_paused());
    assert!(!transport.is_playing());
    assert_eq!(transport.state(), TransportState::Paused);

    // Stepping while paused must NOT advance time
    let state = transport.step(Duration::from_millis(500));
    assert_eq!(state, ClockState::Active);
    assert_eq!(transport.time(), Duration::ZERO);
    assert_eq!(transport.progress(), 0.0);

    // Resuming playback
    transport.play();
    assert!(transport.is_playing());
    assert!(!transport.is_paused());
    assert_eq!(transport.state(), TransportState::Playing);

    let state = transport.step(Duration::from_millis(500));
    assert_eq!(state, ClockState::Active);
    assert_eq!(transport.time(), Duration::from_millis(500));
}

#[test]
fn test_transport_toggle() {
    let mut transport = TimelineTransport::new(Duration::from_secs(1));
    assert!(transport.is_playing());

    // Toggle 1: playing -> paused
    transport.toggle();
    assert!(transport.is_paused());
    assert!(!transport.is_playing());

    transport.step(Duration::from_millis(200));
    assert_eq!(transport.time(), Duration::ZERO);

    // Toggle 2: paused -> playing
    transport.toggle();
    assert!(transport.is_playing());
    assert!(!transport.is_paused());

    transport.step(Duration::from_millis(200));
    assert_eq!(transport.time(), Duration::from_millis(200));
}

#[test]
fn test_transport_time_scale_speed_warping() {
    let mut transport = TimelineTransport::new(Duration::from_secs(10));
    assert_eq!(transport.time_scale(), 1.0);

    // Normal speed (1.0): 100ms -> 100ms
    transport.step(Duration::from_millis(100));
    assert_eq!(transport.time(), Duration::from_millis(100));

    // Double speed (2.0): 100ms -> 200ms increment
    transport.set_time_scale(2.0);
    assert_eq!(transport.time_scale(), 2.0);
    transport.step(Duration::from_millis(100));
    assert_eq!(transport.time(), Duration::from_millis(300));

    // Half speed (0.5): 200ms -> 100ms increment
    transport.set_time_scale(0.5);
    transport.step(Duration::from_millis(200));
    assert_eq!(transport.time(), Duration::from_millis(400));

    // Frozen speed (0.0): 500ms -> 0ms increment
    transport.set_time_scale(0.0);
    transport.step(Duration::from_millis(500));
    assert_eq!(transport.time(), Duration::from_millis(400));
}

#[test]
fn test_transport_reverse_and_direction() {
    let mut transport = TimelineTransport::new(Duration::from_secs(4));

    transport.step(Duration::from_secs(2));
    assert_eq!(transport.time(), Duration::from_secs(2));

    transport.reverse();
    assert!(transport.direction().is_backward());

    // Stepping in backward direction subtracts elapsed time
    transport.step(Duration::from_millis(500));
    assert_eq!(transport.time(), Duration::from_millis(1500));

    // Reverse until 0 reaches completion
    let state = transport.step(Duration::from_millis(1500));
    assert_eq!(state, ClockState::Completed);
    assert_eq!(transport.time(), Duration::ZERO);
    assert!(transport.is_completed());
    assert!(!transport.is_playing());
    assert_eq!(transport.state(), TransportState::Settled);
}

#[test]
fn test_transport_seek_and_progress() {
    let mut transport = TimelineTransport::new(Duration::from_secs(4));

    transport.seek(Duration::from_secs(3));
    assert_eq!(transport.time(), Duration::from_secs(3));
    assert!((transport.progress() - 0.75).abs() < 1e-4);

    transport.set_progress(0.25);
    assert_eq!(transport.time(), Duration::from_secs(1));
    assert!((transport.progress() - 0.25).abs() < 1e-4);

    // Clamping to [0.0, 1.0]
    transport.set_progress(1.5);
    assert_eq!(transport.time(), Duration::from_secs(4));
    assert!((transport.progress() - 1.0).abs() < 1e-4);

    transport.set_progress(-0.5);
    assert_eq!(transport.time(), Duration::ZERO);
    assert!((transport.progress() - 0.0).abs() < 1e-4);
}

#[test]
fn test_transport_completion_and_restart() {
    let mut transport = TimelineTransport::new(Duration::from_secs(1));

    let state = transport.step(Duration::from_secs(1));
    assert_eq!(state, ClockState::Completed);
    assert!(transport.is_completed());
    assert!(!transport.is_playing());
    assert_eq!(transport.state(), TransportState::Settled);

    // Restart resets time to 0, sets direction Forward, and plays
    transport.restart();
    assert!(transport.is_playing());
    assert!(!transport.is_completed());
    assert_eq!(transport.time(), Duration::ZERO);
    assert_eq!(transport.state(), TransportState::Playing);

    // Step again to complete
    transport.step(Duration::from_secs(1));
    assert!(transport.is_completed());

    // Calling play() when completed at end also auto-restarts to beginning
    transport.play();
    assert!(transport.is_playing());
    assert_eq!(transport.time(), Duration::ZERO);
}

#[test]
fn test_high_level_timeline_coordinator_transport_delegation() {
    let mut builder = TimelineBuilder::new();
    builder.tween(
        "x",
        0.0f64,
        100.0f64,
        Duration::from_secs(2),
        Ease::Linear,
        Position::Absolute(Duration::ZERO),
    );

    let mut timeline = Timeline::new(builder.compile().unwrap());
    assert_eq!(timeline.duration(), Duration::from_secs(2));
    assert!(timeline.is_playing());

    // Sample at t = 0
    assert_eq!(timeline.sample::<f64>("x"), Some(0.0));

    // Step 1s
    let state = timeline.step(Duration::from_secs(1));
    assert_eq!(state, ClockState::Active);
    assert_eq!(timeline.time(), Duration::from_secs(1));
    assert_eq!(timeline.sample::<f64>("x"), Some(50.0));

    // Pause
    timeline.pause();
    assert!(timeline.is_paused());
    timeline.step(Duration::from_millis(500));
    assert_eq!(timeline.sample::<f64>("x"), Some(50.0)); // Unchanged

    // Seek directly
    timeline.seek(Duration::from_millis(1500));
    assert_eq!(timeline.sample::<f64>("x"), Some(75.0));

    // Set progress
    timeline.set_progress(1.0);
    assert_eq!(timeline.sample::<f64>("x"), Some(100.0));

    // Reverse
    timeline.reverse();
    timeline.play();
    timeline.step(Duration::from_secs(1));
    assert_eq!(timeline.sample::<f64>("x"), Some(50.0));
}
