//! Target contract for readable and animatable motion targets.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

/// A contract for reading/sampling the current value of an animatable target.
///
/// Any type that can provide a current value of type `T` can implement this trait,
/// enabling dynamic endpoints for `to()` and `from()` tweens without requiring
/// eager value pairing at construction time.
pub trait Target<T> {
    /// Read or sample the current value of the target.
    fn sample(&self) -> T;
}

// Blanket implementation for closures: Fn() -> T
impl<T, F> Target<T> for F
where
    F: Fn() -> T,
{
    #[inline]
    fn sample(&self) -> T {
        self()
    }
}

// Standard library container implementations
impl<T: Copy> Target<T> for Cell<T> {
    #[inline]
    fn sample(&self) -> T {
        self.get()
    }
}

impl<T: Clone> Target<T> for RefCell<T> {
    #[inline]
    fn sample(&self) -> T {
        self.borrow().clone()
    }
}

impl<T: Clone> Target<T> for RwLock<T> {
    #[inline]
    fn sample(&self) -> T {
        self.read().expect("RwLock read poisoned").clone()
    }
}

impl<T: Clone> Target<T> for Mutex<T> {
    #[inline]
    fn sample(&self) -> T {
        self.lock().expect("Mutex lock poisoned").clone()
    }
}

/// A reference-counted dynamic target sampler.
///
/// Wraps an arbitrary dynamic target in an `Rc<dyn Fn() -> T>`,
/// allowing it to be stored inside [`crate::tween::Tween`] while remaining clonable
/// and compatible with UI targets (such as signals, cells, and refcells).
#[derive(Clone)]
pub struct TargetSampler<T> {
    sampler: Rc<dyn Fn() -> T>,
}

impl<T> TargetSampler<T> {
    /// Create a new target sampler from a closure.
    pub fn new(sampler: impl Fn() -> T + 'static) -> Self {
        Self {
            sampler: Rc::new(sampler),
        }
    }
}

impl<T> Target<T> for TargetSampler<T> {
    #[inline]
    fn sample(&self) -> T {
        (self.sampler)()
    }
}

impl<T> fmt::Debug for TargetSampler<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TargetSampler").finish_non_exhaustive()
    }
}

/// Conversion trait to produce a [`TargetSampler<T>`] from various target types.
pub trait IntoTargetSampler<T> {
    /// Convert the target into a [`TargetSampler<T>`].
    fn into_sampler(self) -> TargetSampler<T>;
}

impl<T: 'static> IntoTargetSampler<T> for TargetSampler<T> {
    #[inline]
    fn into_sampler(self) -> TargetSampler<T> {
        self
    }
}

impl<T: 'static, F> IntoTargetSampler<T> for F
where
    F: Fn() -> T + 'static,
{
    #[inline]
    fn into_sampler(self) -> TargetSampler<T> {
        TargetSampler::new(self)
    }
}

impl<T: Copy + 'static> IntoTargetSampler<T> for Rc<Cell<T>> {
    #[inline]
    fn into_sampler(self) -> TargetSampler<T> {
        TargetSampler::new(move || self.get())
    }
}

impl<T: Clone + 'static> IntoTargetSampler<T> for Rc<RefCell<T>> {
    #[inline]
    fn into_sampler(self) -> TargetSampler<T> {
        TargetSampler::new(move || self.borrow().clone())
    }
}

impl<T: Clone + 'static> IntoTargetSampler<T> for Arc<RwLock<T>> {
    #[inline]
    fn into_sampler(self) -> TargetSampler<T> {
        TargetSampler::new(move || self.read().expect("RwLock poisoned").clone())
    }
}

impl<T: Clone + 'static> IntoTargetSampler<T> for Arc<Mutex<T>> {
    #[inline]
    fn into_sampler(self) -> TargetSampler<T> {
        TargetSampler::new(move || self.lock().expect("Mutex poisoned").clone())
    }
}
