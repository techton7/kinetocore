//! Keyframe structures and monotonicity validation for non-uniform timeline clips.

use crate::ease::Ease;
use crate::interpolate::Interpolate;
use crate::timeline::error::TimelineError;

/// A single keyframe point along a normalized interval `[0.0, 1.0]`.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe<T: Interpolate> {
    /// Normalized temporal offset within `[0.0, 1.0]`.
    pub offset: f32,
    /// Value at this keyframe timestamp.
    pub value: T,
    /// Easing curve applied between this keyframe and the next keyframe.
    pub ease: Ease,
}

impl<T: Interpolate> Keyframe<T> {
    /// Create a new keyframe with explicit offset, target value, and easing curve.
    #[inline]
    pub fn new(offset: f32, value: T, ease: Ease) -> Self {
        Self { offset, value, ease }
    }

    /// Create a keyframe with linear easing.
    #[inline]
    pub fn linear(offset: f32, value: T) -> Self {
        Self::new(offset, value, Ease::Linear)
    }
}

/// Validate that a sequence of keyframes satisfies boundary and monotonicity invariants:
///
/// 1. At least 2 keyframes must be present.
/// 2. First keyframe offset must equal `0.0` (within epsilon `1e-5`).
/// 3. Last keyframe offset must equal `1.0` (within epsilon `1e-5`).
/// 4. Keyframe offsets must be strictly monotonically increasing (`offset[i] < offset[i+1]`).
pub fn validate_keyframes<T: Interpolate>(keyframes: &[Keyframe<T>]) -> Result<(), TimelineError> {
    if keyframes.len() < 2 {
        return Err(TimelineError::InvalidKeyframe(format!(
            "keyframe sequence must contain at least 2 keyframes, found {}",
            keyframes.len()
        )));
    }

    let first = &keyframes[0];
    if (first.offset - 0.0).abs() > 1e-5 {
        return Err(TimelineError::InvalidKeyframe(format!(
            "first keyframe offset must be 0.0, found {}",
            first.offset
        )));
    }

    let last = &keyframes[keyframes.len() - 1];
    if (last.offset - 1.0).abs() > 1e-5 {
        return Err(TimelineError::InvalidKeyframe(format!(
            "last keyframe offset must be 1.0, found {}",
            last.offset
        )));
    }

    for i in 0..keyframes.len() - 1 {
        let curr = &keyframes[i];
        let next = &keyframes[i + 1];
        if curr.offset >= next.offset {
            return Err(TimelineError::InvalidKeyframe(format!(
                "keyframes must be strictly monotonic: keyframe[{}] offset ({}) >= keyframe[{}] offset ({})",
                i, curr.offset, i + 1, next.offset
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_keyframes() {
        let kfs = vec![
            Keyframe::linear(0.0, 0.0f32),
            Keyframe::new(0.5, 50.0f32, Ease::QuadInOut),
            Keyframe::linear(1.0, 100.0f32),
        ];
        assert!(validate_keyframes(&kfs).is_ok());
    }

    #[test]
    fn test_too_few_keyframes() {
        let kfs = vec![Keyframe::linear(0.0, 0.0f32)];
        assert!(validate_keyframes(&kfs).is_err());
    }

    #[test]
    fn test_invalid_start_offset() {
        let kfs = vec![
            Keyframe::linear(0.1, 0.0f32),
            Keyframe::linear(1.0, 100.0f32),
        ];
        assert!(validate_keyframes(&kfs).is_err());
    }

    #[test]
    fn test_invalid_end_offset() {
        let kfs = vec![
            Keyframe::linear(0.0, 0.0f32),
            Keyframe::linear(0.9, 100.0f32),
        ];
        assert!(validate_keyframes(&kfs).is_err());
    }

    #[test]
    fn test_duplicate_offsets_rejected() {
        let kfs = vec![
            Keyframe::linear(0.0, 0.0f32),
            Keyframe::linear(0.5, 20.0f32),
            Keyframe::linear(0.5, 50.0f32),
            Keyframe::linear(1.0, 100.0f32),
        ];
        assert!(validate_keyframes(&kfs).is_err());
    }

    #[test]
    fn test_out_of_order_offsets_rejected() {
        let kfs = vec![
            Keyframe::linear(0.0, 0.0f32),
            Keyframe::linear(0.7, 50.0f32),
            Keyframe::linear(0.3, 20.0f32),
            Keyframe::linear(1.0, 100.0f32),
        ];
        assert!(validate_keyframes(&kfs).is_err());
    }
}
