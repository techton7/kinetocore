//! Playback direction definitions.

/// Direction of animation playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackDirection {
    /// Play forward from start toward end.
    #[default]
    Forward,

    /// Play backward from end toward start.
    Backward,
}

impl PlaybackDirection {
    /// Returns `true` if playing forward.
    #[inline]
    pub fn is_forward(&self) -> bool {
        matches!(self, Self::Forward)
    }

    /// Returns `true` if playing backward.
    #[inline]
    pub fn is_backward(&self) -> bool {
        matches!(self, Self::Backward)
    }

    /// Toggle direction between `Forward` and `Backward`.
    #[inline]
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        };
    }
}

impl std::ops::Not for PlaybackDirection {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_basics() {
        let mut dir = PlaybackDirection::Forward;
        assert!(dir.is_forward());
        assert!(!dir.is_backward());

        dir.toggle();
        assert!(dir.is_backward());
        assert!(!dir.is_forward());

        assert_eq!(!PlaybackDirection::Forward, PlaybackDirection::Backward);
        assert_eq!(!PlaybackDirection::Backward, PlaybackDirection::Forward);
    }
}
