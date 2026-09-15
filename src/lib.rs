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
//! ### Core Features
//!
//! - **Dual Mathematics Engine**:
//!   - Industry-standard **Robert Penner Easing** functions via [`easer`].
//!   - Velocity-based **Damped Harmonic Oscillator** (Spring Physics) for natural, interruptible motion.
//! - **Multi-Track Timeline**: First-class scrubbing (`seek`, `progress`, `reverse`, `time_scale`).
//! - **Target-Agnostic Interpolation**: Interpolates any numeric type (`f32`, `f64`, `[f32; N]`, vectors).

#![warn(missing_docs)]

/// Re-export easing functions from easer.
pub use easer::functions as easing;

/// Early scaffold version of kinetocore.
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
}
