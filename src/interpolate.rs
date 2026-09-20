//! Core interpolation contract and implementations for numeric types.

/// A trait for types that can be linearly interpolated between two values.
pub trait Interpolate: Clone {
    /// Linearly interpolate between `self` (at `ratio == 0.0`) and `target` (at `ratio == 1.0`).
    ///
    /// Note that `ratio` may occasionally be less than `0.0` or greater than `1.0`
    /// when using overshooting easing curves (such as [`crate::ease::Ease::BackOut`] or
    /// [`crate::ease::Ease::ElasticOut`]). Implementations should gracefully evaluate
    /// these out-of-bounds ratios using standard extrapolation math.
    fn interpolate(&self, target: &Self, ratio: f32) -> Self;
}

/// Helper function to linearly interpolate between two values.
#[inline]
pub fn lerp<T: Interpolate>(from: &T, to: &T, ratio: f32) -> T {
    from.interpolate(to, ratio)
}

impl Interpolate for f32 {
    #[inline]
    fn interpolate(&self, target: &Self, ratio: f32) -> Self {
        self + (target - self) * ratio
    }
}

impl Interpolate for f64 {
    #[inline]
    fn interpolate(&self, target: &Self, ratio: f32) -> Self {
        self + (target - self) * (ratio as f64)
    }
}

impl<T: Interpolate, const N: usize> Interpolate for [T; N] {
    #[inline]
    fn interpolate(&self, target: &Self, ratio: f32) -> Self {
        let mut result = self.clone();
        for i in 0..N {
            result[i] = self[i].interpolate(&target[i], ratio);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_interpolation() {
        let a = 10.0f32;
        let b = 20.0f32;

        assert_eq!(a.interpolate(&b, 0.0), 10.0);
        assert_eq!(a.interpolate(&b, 0.5), 15.0);
        assert_eq!(a.interpolate(&b, 1.0), 20.0);

        // Overshoot tests
        assert_eq!(a.interpolate(&b, 1.5), 25.0);
        assert_eq!(a.interpolate(&b, -0.5), 5.0);
    }

    #[test]
    fn test_f64_interpolation() {
        let a = 0.0f64;
        let b = 100.0f64;

        assert_eq!(a.interpolate(&b, 0.25), 25.0);
        assert_eq!(a.interpolate(&b, 0.75), 75.0);
    }

    #[test]
    fn test_array_interpolation() {
        let a2 = [0.0f32, 10.0f32];
        let b2 = [10.0f32, 30.0f32];
        assert_eq!(a2.interpolate(&b2, 0.5), [5.0, 20.0]);

        let a3 = [0.0f32, 100.0f32, -50.0f32];
        let b3 = [10.0f32, 200.0f32, 50.0f32];
        assert_eq!(a3.interpolate(&b3, 0.5), [5.0, 150.0, 0.0]);

        let a4 = [1.0f32, 2.0, 3.0, 4.0];
        let b4 = [2.0f32, 4.0, 6.0, 8.0];
        assert_eq!(a4.interpolate(&b4, 0.25), [1.25, 2.5, 3.75, 5.0]);
    }

    #[test]
    fn test_lerp_helper() {
        assert_eq!(lerp(&10.0f32, &20.0f32, 0.5), 15.0f32);
    }
}
