use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use kinetocore::prelude::*;

#[test]
fn test_to_tween_lazy_init_and_latching() {
    let current_x = Rc::new(Cell::new(10.0f32));
    let x_sampler = current_x.clone();

    // Create a tween animating to 100.0 over 10s
    let mut tween = Tween::to(move || x_sampler.get(), 100.0f32, Duration::from_secs(10))
        .ease(Ease::Linear);

    // Initial state: not yet latched
    assert!(!tween.is_initialized());
    assert_eq!(tween.from_value(), None);
    assert_eq!(tween.to_value(), Some(&100.0));

    // Dynamic mutation before first step alters the preview value
    current_x.set(20.0);
    assert_eq!(tween.value(), 20.0);

    // First step (at 2s = 20%): latches start point to 20.0
    // Range is 20.0 to 100.0 (delta 80.0), 20% = 20.0 + 16.0 = 36.0
    let (val, state) = tween.step(Duration::from_secs(2));
    assert_eq!(val, 36.0);
    assert_eq!(state, ClockState::Active);
    assert!(tween.is_initialized());
    assert_eq!(tween.from_value(), Some(&20.0));
    assert_eq!(tween.to_value(), Some(&100.0));

    // External target mutation during active playback MUST NOT disrupt latched trajectory
    current_x.set(999.0);
    let (val, state) = tween.step(Duration::from_secs(3)); // at 5s = 50% = 20.0 + 40.0 = 60.0
    assert_eq!(val, 60.0);
    assert_eq!(state, ClockState::Active);

    // Run to completion (5s more = 100%)
    let (val, state) = tween.step(Duration::from_secs(5));
    assert_eq!(val, 100.0);
    assert_eq!(state, ClockState::Completed);
    assert!(tween.is_completed());
}

#[test]
fn test_from_tween_lazy_init_and_latching() {
    let current_opacity = Arc::new(RwLock::new(1.0f32));
    let op_clone = current_opacity.clone();

    // Animate from 0.0 to whatever the target currently is (fade in to current)
    let mut tween = Tween::from(move || *op_clone.read().unwrap(), 0.0f32, Duration::from_secs(4))
        .ease(Ease::Linear);

    assert!(!tween.is_initialized());
    assert_eq!(tween.from_value(), Some(&0.0));
    assert_eq!(tween.to_value(), None);

    // Target adjusted before playback begins
    *current_opacity.write().unwrap() = 0.8f32;

    // Step 1s (25%): latches destination to 0.8
    // Range 0.0 to 0.8, 25% = 0.2
    let (val, state) = tween.step(Duration::from_secs(1));
    assert_eq!(val, 0.2);
    assert_eq!(state, ClockState::Active);
    assert!(tween.is_initialized());
    assert_eq!(tween.to_value(), Some(&0.8));

    // Target change after latching has no effect
    *current_opacity.write().unwrap() = 0.0;
    let (val, _) = tween.step(Duration::from_secs(3));
    assert_eq!(val, 0.8);
    assert!(tween.is_completed());
}

#[test]
fn test_explicit_ensure_initialized() {
    let counter = Arc::new(AtomicU64::new(50));
    let c = counter.clone();

    let mut tween = Tween::to(
        move || c.load(Ordering::Relaxed) as f64,
        150.0f64,
        Duration::from_secs(2),
    );

    assert!(!tween.is_initialized());
    tween.ensure_initialized();
    assert!(tween.is_initialized());
    assert_eq!(tween.from_value(), Some(&50.0));
    assert_eq!(tween.to_value(), Some(&150.0));

    // Calling ensure_initialized multiple times is safe and idempotent
    counter.store(999, Ordering::Relaxed);
    tween.ensure_initialized();
    assert_eq!(tween.from_value(), Some(&50.0));
}

#[test]
fn test_target_with_mutex_and_array() {
    let array_target = Arc::new(Mutex::new([0.0f32, 10.0, 50.0]));
    let t_clone = array_target.clone();

    let mut tween = Tween::to(
        move || *t_clone.lock().unwrap(),
        [100.0f32, 50.0, 250.0],
        Duration::from_secs(2),
    )
    .ease(Ease::Linear);

    // 50% = 1 sec
    let (v, state) = tween.step(Duration::from_secs(1));
    assert_eq!(v, [50.0, 30.0, 150.0]);
    assert_eq!(state, ClockState::Active);

    // 100% = 2 sec
    let (v, state) = tween.step(Duration::from_secs(1));
    assert_eq!(v, [100.0, 50.0, 250.0]);
    assert_eq!(state, ClockState::Completed);
}
