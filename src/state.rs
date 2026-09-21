//! State representation and lazy initialization foundation for tweens.

use std::fmt;

use crate::target::{Target, TargetSampler};

/// Represents the endpoints of a tween, either eagerly initialized or pending lazy target sampling.
#[derive(Clone)]
pub enum TweenEndpoints<T> {
    /// Both start and end values are concrete and ready for immediate evaluation.
    Ready {
        /// Starting value of the tween.
        from: T,
        /// Ending value of the tween.
        to: T,
    },
    /// A `to` tween whose starting value will be sampled lazily from `sampler` on the first tick.
    PendingTo {
        /// Target sampler to read starting value from.
        sampler: TargetSampler<T>,
        /// Concrete target destination value.
        to: T,
    },
    /// A `from` tween whose destination value will be sampled lazily from `sampler` on the first tick.
    PendingFrom {
        /// Target sampler to read destination value from.
        sampler: TargetSampler<T>,
        /// Concrete starting value.
        from: T,
    },
}

impl<T: Clone> TweenEndpoints<T> {
    /// Create an eagerly initialized endpoints pair.
    #[inline]
    pub fn eager(from: T, to: T) -> Self {
        Self::Ready { from, to }
    }

    /// Create a lazy `to` endpoint pair pending target sampling.
    #[inline]
    pub fn lazy_to(sampler: TargetSampler<T>, to: T) -> Self {
        Self::PendingTo { sampler, to }
    }

    /// Create a lazy `from` endpoint pair pending target sampling.
    #[inline]
    pub fn lazy_from(sampler: TargetSampler<T>, from: T) -> Self {
        Self::PendingFrom { sampler, from }
    }

    /// Returns `true` if the endpoints have been fully resolved into concrete start/end values.
    #[inline]
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    /// Resolves and latches the endpoints if they are currently pending.
    ///
    /// If in `PendingTo`, samples the start value and converts to `Ready`.
    /// If in `PendingFrom`, samples the end value and converts to `Ready`.
    /// Once latched into `Ready`, subsequent target changes will not alter these endpoints.
    pub fn resolve(&mut self) -> (&T, &T) {
        match self {
            Self::Ready { from, to } => (from, to),
            Self::PendingTo { sampler, to } => {
                let from = sampler.sample();
                let dest = to.clone();
                *self = Self::Ready { from, to: dest };
                match self {
                    Self::Ready { from, to } => (from, to),
                    _ => unreachable!(),
                }
            }
            Self::PendingFrom { sampler, from } => {
                let to = sampler.sample();
                let start = from.clone();
                *self = Self::Ready { from: start, to };
                match self {
                    Self::Ready { from, to } => (from, to),
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Samples the current endpoints without permanently mutating internal state.
    ///
    /// Useful for non-mutating preview or inspection before the animation officially starts ticking.
    pub fn sample_current_endpoints(&self) -> (T, T) {
        match self {
            Self::Ready { from, to } => (from.clone(), to.clone()),
            Self::PendingTo { sampler, to } => (sampler.sample(), to.clone()),
            Self::PendingFrom { sampler, from } => (from.clone(), sampler.sample()),
        }
    }

    /// Returns a reference to the start value if it is already known or latched.
    #[inline]
    pub fn from_value(&self) -> Option<&T> {
        match self {
            Self::Ready { from, .. } | Self::PendingFrom { from, .. } => Some(from),
            Self::PendingTo { .. } => None,
        }
    }

    /// Returns a reference to the end value if it is already known or latched.
    #[inline]
    pub fn to_value(&self) -> Option<&T> {
        match self {
            Self::Ready { to, .. } | Self::PendingTo { to, .. } => Some(to),
            Self::PendingFrom { .. } => None,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for TweenEndpoints<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ready { from, to } => f
                .debug_struct("Ready")
                .field("from", from)
                .field("to", to)
                .finish(),
            Self::PendingTo { to, .. } => f
                .debug_struct("PendingTo")
                .field("to", to)
                .finish_non_exhaustive(),
            Self::PendingFrom { from, .. } => f
                .debug_struct("PendingFrom")
                .field("from", from)
                .finish_non_exhaustive(),
        }
    }
}

impl<T: PartialEq> PartialEq for TweenEndpoints<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ready { from: f1, to: t1 }, Self::Ready { from: f2, to: t2 }) => {
                f1 == f2 && t1 == t2
            }
            (Self::PendingTo { to: t1, .. }, Self::PendingTo { to: t2, .. }) => t1 == t2,
            (Self::PendingFrom { from: f1, .. }, Self::PendingFrom { from: f2, .. }) => f1 == f2,
            _ => false,
        }
    }
}
