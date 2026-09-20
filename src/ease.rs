//! Easing curve definitions and evaluation wrapper around Robert Penner easing equations.

use easer::functions::*;

/// Standard easing curves for interpolation.
///
/// Wraps the Robert Penner mathematical easing equations into a unified,
/// idiomatic Rust enum that evaluates normalized progress `ratio` in `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ease {
    /// Linear progress with no acceleration or deceleration.
    #[default]
    Linear,

    /// Quadratic acceleration (`t^2`).
    QuadIn,
    /// Quadratic deceleration.
    QuadOut,
    /// Quadratic acceleration then deceleration.
    QuadInOut,

    /// Cubic acceleration (`t^3`).
    CubicIn,
    /// Cubic deceleration.
    CubicOut,
    /// Cubic acceleration then deceleration.
    CubicInOut,

    /// Quartic acceleration (`t^4`).
    QuartIn,
    /// Quartic deceleration.
    QuartOut,
    /// Quartic acceleration then deceleration.
    QuartInOut,

    /// Quintic acceleration (`t^5`).
    QuintIn,
    /// Quintic deceleration.
    QuintOut,
    /// Quintic acceleration then deceleration.
    QuintInOut,

    /// Sinusoidal acceleration.
    SineIn,
    /// Sinusoidal deceleration.
    SineOut,
    /// Sinusoidal acceleration then deceleration.
    SineInOut,

    /// Circular acceleration.
    CircIn,
    /// Circular deceleration.
    CircOut,
    /// Circular acceleration then deceleration.
    CircInOut,

    /// Exponential acceleration.
    ExpoIn,
    /// Exponential deceleration.
    ExpoOut,
    /// Exponential acceleration then deceleration.
    ExpoInOut,

    /// Elastic acceleration resembling a spring pulled back.
    ElasticIn,
    /// Elastic deceleration resembling an oscillating spring settling.
    ElasticOut,
    /// Elastic acceleration and deceleration.
    ElasticInOut,

    /// Backing up slightly before moving forward.
    BackIn,
    /// Overshooting slightly before settling at destination.
    BackOut,
    /// Backing up then overshooting before settling.
    BackInOut,

    /// Bouncing acceleration.
    BounceIn,
    /// Bouncing deceleration.
    BounceOut,
    /// Bouncing acceleration then deceleration.
    BounceInOut,
}

impl Ease {
    /// Sample the easing curve at a normalized progress `ratio`.
    ///
    /// The input `ratio` is typically in `[0.0, 1.0]`.
    /// For standard curves, `sample(0.0) == 0.0` and `sample(1.0) == 1.0`.
    #[inline]
    pub fn sample(&self, ratio: f32) -> f32 {
        const B: f32 = 0.0;
        const C: f32 = 1.0;
        const D: f32 = 1.0;

        match self {
            Ease::Linear => Linear::ease_in(ratio, B, C, D),

            Ease::QuadIn => Quad::ease_in(ratio, B, C, D),
            Ease::QuadOut => Quad::ease_out(ratio, B, C, D),
            Ease::QuadInOut => Quad::ease_in_out(ratio, B, C, D),

            Ease::CubicIn => Cubic::ease_in(ratio, B, C, D),
            Ease::CubicOut => Cubic::ease_out(ratio, B, C, D),
            Ease::CubicInOut => Cubic::ease_in_out(ratio, B, C, D),

            Ease::QuartIn => Quart::ease_in(ratio, B, C, D),
            Ease::QuartOut => Quart::ease_out(ratio, B, C, D),
            Ease::QuartInOut => Quart::ease_in_out(ratio, B, C, D),

            Ease::QuintIn => Quint::ease_in(ratio, B, C, D),
            Ease::QuintOut => Quint::ease_out(ratio, B, C, D),
            Ease::QuintInOut => Quint::ease_in_out(ratio, B, C, D),

            Ease::SineIn => Sine::ease_in(ratio, B, C, D),
            Ease::SineOut => Sine::ease_out(ratio, B, C, D),
            Ease::SineInOut => Sine::ease_in_out(ratio, B, C, D),

            Ease::CircIn => Circ::ease_in(ratio, B, C, D),
            Ease::CircOut => Circ::ease_out(ratio, B, C, D),
            Ease::CircInOut => Circ::ease_in_out(ratio, B, C, D),

            Ease::ExpoIn => Expo::ease_in(ratio, B, C, D),
            Ease::ExpoOut => Expo::ease_out(ratio, B, C, D),
            Ease::ExpoInOut => Expo::ease_in_out(ratio, B, C, D),

            Ease::ElasticIn => Elastic::ease_in(ratio, B, C, D),
            Ease::ElasticOut => Elastic::ease_out(ratio, B, C, D),
            Ease::ElasticInOut => Elastic::ease_in_out(ratio, B, C, D),

            Ease::BackIn => Back::ease_in(ratio, B, C, D),
            Ease::BackOut => Back::ease_out(ratio, B, C, D),
            Ease::BackInOut => Back::ease_in_out(ratio, B, C, D),

            Ease::BounceIn => Bounce::ease_in(ratio, B, C, D),
            Ease::BounceOut => Bounce::ease_out(ratio, B, C, D),
            Ease::BounceInOut => Bounce::ease_in_out(ratio, B, C, D),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_EASES: [Ease; 31] = [
        Ease::Linear,
        Ease::QuadIn,
        Ease::QuadOut,
        Ease::QuadInOut,
        Ease::CubicIn,
        Ease::CubicOut,
        Ease::CubicInOut,
        Ease::QuartIn,
        Ease::QuartOut,
        Ease::QuartInOut,
        Ease::QuintIn,
        Ease::QuintOut,
        Ease::QuintInOut,
        Ease::SineIn,
        Ease::SineOut,
        Ease::SineInOut,
        Ease::CircIn,
        Ease::CircOut,
        Ease::CircInOut,
        Ease::ExpoIn,
        Ease::ExpoOut,
        Ease::ExpoInOut,
        Ease::ElasticIn,
        Ease::ElasticOut,
        Ease::ElasticInOut,
        Ease::BackIn,
        Ease::BackOut,
        Ease::BackInOut,
        Ease::BounceIn,
        Ease::BounceOut,
        Ease::BounceInOut,
    ];

    #[test]
    fn test_endpoints() {
        for ease in ALL_EASES {
            let start = ease.sample(0.0);
            let end = ease.sample(1.0);

            assert!(
                start.abs() < 1e-4,
                "Ease::{:?} failed start endpoint: got {}",
                ease,
                start
            );
            assert!(
                (end - 1.0).abs() < 1e-4,
                "Ease::{:?} failed end endpoint: got {}",
                ease,
                end
            );
        }
    }

    #[test]
    fn test_midpoints() {
        // QuadInOut at 0.5 should be exactly 0.5
        let mid = Ease::QuadInOut.sample(0.5);
        assert!((mid - 0.5).abs() < 1e-4);

        // Linear at 0.5 should be exactly 0.5
        let mid_linear = Ease::Linear.sample(0.5);
        assert!((mid_linear - 0.5).abs() < 1e-4);

        // QuadIn at 0.5 is 0.5^2 = 0.25
        let mid_quad_in = Ease::QuadIn.sample(0.5);
        assert!((mid_quad_in - 0.25).abs() < 1e-4);
    }
}
