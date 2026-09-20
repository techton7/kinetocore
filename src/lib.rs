//! # Kinetocore
//!
//! Pure Rust, framework-agnostic physics and multi-track timeline motion engine.
//!
//! ## Overview
//!
//! `kinetocore` is a headless, universal motion and timeline sequencing engine with **zero UI dependencies**.
//! Operating with nanosecond numerical precision, it powers fluid animations, spring physics,
//! and multi-track timeline orchestrations across:
//! - **WGPU & 3D Viewports**: Direct GPU buffer updates with zero UI overhead.
//! - **Dioxus Ecosystem**: Serves as the foundational mathematical engine behind `kinetoxus`.
//! - **Universal Rust Applications**: Usable in Bevy, Slint, egui, or headless rendering pipelines.
//!
//! ### Core Pillars
//!
//! - **[`Interpolate`]**: Atomic value-level linear interpolation for scalars and arrays.
//! - **[`Ease`]**: Unified enum wrapping Robert Penner easing formulas via [`easer`].
//! - **[`AnimClock`]**: Timekeeping, repeat cycles, and yoyo/mirrored arithmetic.
//! - **[`Tween`]**: Pure-value tweening supporting `set` and `from_to`.

#![warn(missing_docs)]

pub mod clock;
pub mod direction;
pub mod ease;
pub mod interpolate;
pub mod prelude;
pub mod repeat;
pub mod tween;

// Re-export prelude at root level
pub use clock::{AnimClock, ClockState};
pub use direction::PlaybackDirection;
pub use ease::Ease;
pub use interpolate::{lerp, Interpolate};
pub use repeat::{RepeatCount, RepeatStrategy};
pub use tween::Tween;

/// Re-export easing functions from easer for backwards compatibility.
pub use easer::functions as easing;

/// Current package version of kinetocore.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_easing_reexport() {
        use easer::functions::Easing;
        let v = easing::Quad::ease_in_out(0.5f32, 0.0, 100.0, 1.0);
        assert!((v - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_root_reexports() {
        let mut tween = Tween::from_to(0.0f32, 10.0f32, std::time::Duration::from_secs(1))
            .ease(Ease::QuadInOut);
        let (val, state) = tween.step(std::time::Duration::from_millis(500));
        assert!((val - 5.0).abs() < 1e-4);
        assert_eq!(state, ClockState::Active);
    }
}
