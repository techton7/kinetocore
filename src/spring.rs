//! Analytical scalar spring physics kernel.
//!
//! Provides a headless 1D mass-spring-damper motion engine with exact closed-form
//! solutions across underdamped, critically damped, and overdamped regimes.

use std::time::Duration;

/// Configuration parameters for a spring system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    /// Mass ($m > 0$).
    pub mass: f64,
    /// Stiffness constant ($k > 0$).
    pub stiffness: f64,
    /// Damping coefficient ($c \ge 0$).
    pub damping: f64,
}

impl SpringConfig {
    /// Default spring preset (mass = 1.0, stiffness = 100.0, damping = 10.0).
    pub const DEFAULT: Self = Self {
        mass: 1.0,
        stiffness: 100.0,
        damping: 10.0,
    };

    /// Gentle spring preset (mass = 1.0, stiffness = 120.0, damping = 14.0).
    pub const GENTLE: Self = Self {
        mass: 1.0,
        stiffness: 120.0,
        damping: 14.0,
    };

    /// Wobbly spring preset (mass = 1.0, stiffness = 180.0, damping = 12.0).
    pub const WOBBLY: Self = Self {
        mass: 1.0,
        stiffness: 180.0,
        damping: 12.0,
    };

    /// Stiff spring preset (mass = 1.0, stiffness = 210.0, damping = 20.0).
    pub const STIFF: Self = Self {
        mass: 1.0,
        stiffness: 210.0,
        damping: 20.0,
    };

    /// Bouncy spring preset (mass = 1.0, stiffness = 300.0, damping = 8.0).
    pub const BOUNCY: Self = Self {
        mass: 1.0,
        stiffness: 300.0,
        damping: 8.0,
    };

    /// Create and validate a new spring configuration.
    #[inline]
    pub fn new(mass: f64, stiffness: f64, damping: f64) -> Result<Self, SpringError> {
        if mass <= 0.0 || mass.is_nan() {
            return Err(SpringError::InvalidMass(mass));
        }
        if stiffness <= 0.0 || stiffness.is_nan() {
            return Err(SpringError::InvalidStiffness(stiffness));
        }
        if damping < 0.0 || damping.is_nan() {
            return Err(SpringError::InvalidDamping(damping));
        }
        Ok(Self {
            mass,
            stiffness,
            damping,
        })
    }
}

impl Default for SpringConfig {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Errors that can occur when constructing a spring configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpringError {
    /// Mass must be strictly positive ($m > 0$).
    InvalidMass(f64),
    /// Stiffness must be strictly positive ($k > 0$).
    InvalidStiffness(f64),
    /// Damping must be non-negative ($c \ge 0$).
    InvalidDamping(f64),
}

impl std::fmt::Display for SpringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMass(m) => write!(f, "Invalid spring mass {m}: must be strictly positive (> 0.0)"),
            Self::InvalidStiffness(k) => write!(f, "Invalid spring stiffness {k}: must be strictly positive (> 0.0)"),
            Self::InvalidDamping(c) => write!(f, "Invalid spring damping {c}: must be non-negative (>= 0.0)"),
        }
    }
}

impl std::error::Error for SpringError {}

/// The damping regime of a spring system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DampingRegime {
    /// Underdamped ($\zeta < 1.0$): oscillates around equilibrium with decaying amplitude.
    Underdamped,
    /// Critically damped ($\zeta = 1.0$): returns to equilibrium as fast as possible without oscillation.
    CriticallyDamped,
    /// Overdamped ($\zeta > 1.0$): returns to equilibrium sluggishly without oscillating.
    Overdamped,
}

/// Instantaneous state of a spring at time $t$ (position and velocity).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringState {
    /// Current position $x(t)$.
    pub position: f64,
    /// Current velocity $v(t) = dx/dt$.
    pub velocity: f64,
}

/// An analytical scalar spring motion generator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    config: SpringConfig,
    initial_position: f64,
    target_position: f64,
    initial_velocity: f64,
    omega_0: f64,
    zeta: f64,
    regime: DampingRegime,
}

impl Spring {
    /// Boundary epsilon for near-critical damping stability ($|\zeta - 1.0| \le 10^{-5}$).
    pub const NEAR_CRITICAL_EPSILON: f64 = 1e-5;

    /// Create a new analytical spring instance.
    #[inline]
    pub fn new(
        config: SpringConfig,
        initial: f64,
        target: f64,
        initial_velocity: f64,
    ) -> Result<Self, SpringError> {
        if config.mass <= 0.0 || config.mass.is_nan() {
            return Err(SpringError::InvalidMass(config.mass));
        }
        if config.stiffness <= 0.0 || config.stiffness.is_nan() {
            return Err(SpringError::InvalidStiffness(config.stiffness));
        }
        if config.damping < 0.0 || config.damping.is_nan() {
            return Err(SpringError::InvalidDamping(config.damping));
        }

        let omega_0 = (config.stiffness / config.mass).sqrt();
        let zeta = config.damping / (2.0 * (config.mass * config.stiffness).sqrt());

        let regime = if (zeta - 1.0).abs() <= Self::NEAR_CRITICAL_EPSILON {
            DampingRegime::CriticallyDamped
        } else if zeta < 1.0 {
            DampingRegime::Underdamped
        } else {
            DampingRegime::Overdamped
        };

        Ok(Self {
            config,
            initial_position: initial,
            target_position: target,
            initial_velocity,
            omega_0,
            zeta,
            regime,
        })
    }

    /// Returns reference to the spring configuration.
    #[inline]
    pub fn config(&self) -> &SpringConfig {
        &self.config
    }

    /// Returns the initial position $x_0$.
    #[inline]
    pub fn initial_position(&self) -> f64 {
        self.initial_position
    }

    /// Returns the target equilibrium position $x_{target}$.
    #[inline]
    pub fn target_position(&self) -> f64 {
        self.target_position
    }

    /// Returns the initial velocity $v_0$.
    #[inline]
    pub fn initial_velocity(&self) -> f64 {
        self.initial_velocity
    }

    /// Returns the natural angular frequency $\omega_0$.
    #[inline]
    pub fn omega_0(&self) -> f64 {
        self.omega_0
    }

    /// Returns the damping ratio $\zeta$.
    #[inline]
    pub fn zeta(&self) -> f64 {
        self.zeta
    }

    /// Returns the computed damping regime.
    #[inline]
    pub fn regime(&self) -> DampingRegime {
        self.regime
    }

    /// Sample the spring state at duration `t`.
    #[inline]
    pub fn sample(&self, t: Duration) -> SpringState {
        self.sample_secs(t.as_secs_f64())
    }

    /// Sample the spring position at duration `t`.
    #[inline]
    pub fn sample_position(&self, t: Duration) -> f64 {
        self.sample(t).position
    }

    /// Sample the spring velocity at duration `t`.
    #[inline]
    pub fn sample_velocity(&self, t: Duration) -> f64 {
        self.sample(t).velocity
    }

    /// Sample the spring state at time `t` in seconds using exact closed-form analytical solutions.
    pub fn sample_secs(&self, t: f64) -> SpringState {
        if t <= 0.0 {
            return SpringState {
                position: self.initial_position,
                velocity: self.initial_velocity,
            };
        }

        let y0 = self.initial_position - self.target_position;
        let v0 = self.initial_velocity;
        let w0 = self.omega_0;
        let zeta = self.zeta;

        let (y, v) = match self.regime {
            DampingRegime::Underdamped => {
                let wd = w0 * (1.0 - zeta * zeta).sqrt();
                let e_term = (-zeta * w0 * t).exp();
                let cos_term = (wd * t).cos();
                let sin_term = (wd * t).sin();

                let y = e_term * (y0 * cos_term + ((v0 + zeta * w0 * y0) / wd) * sin_term);
                let v = e_term * (v0 * cos_term - ((w0 * w0 * y0 + zeta * w0 * v0) / wd) * sin_term);
                (y, v)
            }
            DampingRegime::CriticallyDamped => {
                let e_term = (-w0 * t).exp();
                let term = y0 + (v0 + w0 * y0) * t;
                let y = e_term * term;
                let v = e_term * (v0 - w0 * (v0 + w0 * y0) * t);
                (y, v)
            }
            DampingRegime::Overdamped => {
                let wd_prime = w0 * (zeta * zeta - 1.0).sqrt();
                let e_term = (-zeta * w0 * t).exp();
                let c1 = y0;
                let c2 = (v0 + zeta * w0 * y0) / wd_prime;
                let cosh_term = (wd_prime * t).cosh();
                let sinh_term = (wd_prime * t).sinh();

                let y = e_term * (c1 * cosh_term + c2 * sinh_term);
                let v = e_term * ((c2 * wd_prime - zeta * w0 * c1) * cosh_term + (c1 * wd_prime - zeta * w0 * c2) * sinh_term);
                (y, v)
            }
        };

        SpringState {
            position: self.target_position + y,
            velocity: v,
        }
    }

    /// Return a new spring instance retargeted to `new_target` at time `t`, maintaining $C^1$ continuity.
    #[inline]
    pub fn retargeted(&self, new_target: f64, t: Duration) -> Self {
        self.retargeted_secs(new_target, t.as_secs_f64())
    }

    /// Return a new spring instance retargeted to `new_target` at time `t` in seconds, maintaining $C^1$ continuity.
    pub fn retargeted_secs(&self, new_target: f64, t: f64) -> Self {
        let state = self.sample_secs(t);
        Self {
            config: self.config,
            initial_position: state.position,
            target_position: new_target,
            initial_velocity: state.velocity,
            omega_0: self.omega_0,
            zeta: self.zeta,
            regime: self.regime,
        }
    }

    /// Retarget the spring to a new target position at time `t`, maintaining $C^1$ continuity
    /// (smooth transition of position and velocity).
    #[inline]
    pub fn retarget(&mut self, new_target: f64, t: Duration) {
        *self = self.retargeted(new_target, t);
    }

    /// Retarget the spring in-place to a new target position at time `t` in seconds, maintaining $C^1$ continuity.
    #[inline]
    pub fn retarget_secs(&mut self, new_target: f64, t: f64) {
        *self = self.retargeted_secs(new_target, t);
    }

    /// Check whether the spring has settled at time `t` within default thresholds ($\epsilon_x = 0.001, \epsilon_v = 0.001$).
    #[inline]
    pub fn settled(&self, t: Duration) -> bool {
        self.settled_with_thresholds(t, 0.001, 0.001)
    }

    /// Check whether the spring has settled at time `t` within custom position and velocity thresholds.
    #[inline]
    pub fn settled_with_thresholds(&self, t: Duration, eps_x: f64, eps_v: f64) -> bool {
        self.is_settled_with_thresholds(t, eps_x, eps_v)
    }

    /// Check whether the spring has settled at time `t` within default thresholds ($\epsilon_x = 0.001, \epsilon_v = 0.001$).
    #[inline]
    pub fn is_settled(&self, t: Duration) -> bool {
        self.is_settled_with_thresholds(t, 0.001, 0.001)
    }

    /// Check whether the spring has settled at time `t` within custom position and velocity thresholds.
    #[inline]
    pub fn is_settled_with_thresholds(&self, t: Duration, eps_x: f64, eps_v: f64) -> bool {
        let state = self.sample(t);
        (state.position - self.target_position).abs() <= eps_x && state.velocity.abs() <= eps_v
    }

    /// Estimate the settle duration of the spring using default thresholds ($\epsilon_x = 0.001, \epsilon_v = 0.001$).
    #[inline]
    pub fn settle_duration(&self) -> Option<Duration> {
        self.settle_duration_with_thresholds(0.001, 0.001)
    }

    /// Estimate the settle duration of the spring using custom thresholds.
    pub fn settle_duration_with_thresholds(&self, eps_x: f64, eps_v: f64) -> Option<Duration> {
        let secs = self.settle_duration_secs_with_thresholds(eps_x, eps_v)?;
        Some(Duration::from_secs_f64(secs))
    }

    /// Estimate the settle duration in seconds using custom thresholds.
    pub fn settle_duration_secs_with_thresholds(&self, eps_x: f64, eps_v: f64) -> Option<f64> {
        // If undamped (c = 0), settle duration does not exist (unless already at target with v=0).
        if self.config.damping == 0.0 {
            let y0 = (self.initial_position - self.target_position).abs();
            let v0 = self.initial_velocity.abs();
            if y0 <= eps_x && v0 <= eps_v {
                return Some(0.0);
            }
            return None;
        }

        let y0 = self.initial_position - self.target_position;
        let v0 = self.initial_velocity;

        // Check if already settled at t = 0
        if y0.abs() <= eps_x && v0.abs() <= eps_v {
            return Some(0.0);
        }

        // Derive an analytical upper bound t_max based on decay rate
        let w0 = self.omega_0;
        let zeta = self.zeta;

        let gamma = match self.regime {
            DampingRegime::Underdamped => zeta * w0,
            DampingRegime::CriticallyDamped => w0,
            DampingRegime::Overdamped => w0 * (zeta - (zeta * zeta - 1.0).sqrt()),
        };

        if gamma <= 0.0 {
            return None;
        }

        // Estimate amplitude bounds to form t_max upper bound
        let t_max = match self.regime {
            DampingRegime::Underdamped => {
                let wd = w0 * (1.0 - zeta * zeta).sqrt();
                // Avoid division by zero if wd is extremely small
                let ax = if wd > 1e-12 {
                    (y0 * y0 + ((v0 + zeta * w0 * y0) / wd).powi(2)).sqrt()
                } else {
                    y0.abs()
                };
                let av = if wd > 1e-12 {
                    (v0 * v0 + ((w0 * w0 * y0 + zeta * w0 * v0) / wd).powi(2)).sqrt()
                } else {
                    v0.abs()
                };

                let t_x = if ax > eps_x { (ax / eps_x).ln() / gamma } else { 0.0 };
                let t_v = if av > eps_v { (av / eps_v).ln() / gamma } else { 0.0 };
                t_x.max(t_v).max(0.0) + 1.0
            }
            DampingRegime::CriticallyDamped | DampingRegime::Overdamped => {
                // For critically damped / overdamped: x(t) = e^(-gamma * t) * (y0 + (v0 + w0 y0) t)
                // To bound safely, take a slightly larger multiplier or solve peak envelope.
                let peak_val = y0.abs() + (v0.abs() / (gamma * 0.5 + 1e-6)).abs();
                let t_approx = if peak_val > eps_x {
                    // Include extra headroom for linear t factor in e^(-gamma * t) * t
                    (peak_val / eps_x).ln() / (gamma * 0.5)
                } else {
                    0.0
                };
                t_approx.max(2.0)
            }
        };

        // Clamp t_max to a safe maximum (e.g. 60.0s)
        let t_max = t_max.min(60.0);

        // If even at t_max it's not settled, return None (or t_max if close)
        if !self.is_settled_with_thresholds(Duration::from_secs_f64(t_max), eps_x, eps_v) {
            // Try larger if needed or return None
            if t_max >= 60.0 {
                return None;
            }
        }

        // Refine via backward search / binary search from 0.0 to t_max
        // Find the exact transition time T where for all t >= T, settled is true.
        // We can do a coarse backward scan with step min(0.01, 0.1 / w0), then binary search.
        let step = (0.01_f64.min(0.1 / w0)).max(1e-4);
        let mut t_found = t_max;
        let mut t_curr = t_max;

        while t_curr >= 0.0 {
            if self.is_settled_with_thresholds(Duration::from_secs_f64(t_curr), eps_x, eps_v) {
                t_found = t_curr;
                t_curr -= step;
            } else {
                break;
            }
        }

        // Binary search between t_found and t_found + step for higher precision (within 0.1ms)
        let mut low = (t_found - step).max(0.0);
        let mut high = t_found;
        for _ in 0..15 {
            let mid = 0.5 * (low + high);
            if self.is_settled_with_thresholds(Duration::from_secs_f64(mid), eps_x, eps_v) {
                high = mid;
            } else {
                low = mid;
            }
        }

        Some(high)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        assert!(SpringConfig::new(1.0, 100.0, 10.0).is_ok());
        assert_eq!(SpringConfig::new(0.0, 100.0, 10.0), Err(SpringError::InvalidMass(0.0)));
        assert_eq!(SpringConfig::new(-1.0, 100.0, 10.0), Err(SpringError::InvalidMass(-1.0)));
        assert_eq!(SpringConfig::new(1.0, 0.0, 10.0), Err(SpringError::InvalidStiffness(0.0)));
        assert_eq!(SpringConfig::new(1.0, -10.0, 10.0), Err(SpringError::InvalidStiffness(-10.0)));
        assert_eq!(SpringConfig::new(1.0, 100.0, -0.1), Err(SpringError::InvalidDamping(-0.1)));
    }

    #[test]
    fn test_t_zero_accuracy() -> Result<(), SpringError> {
        let spring = Spring::new(SpringConfig::DEFAULT, 0.0, 100.0, 5.0)?;
        let state = spring.sample(Duration::ZERO);
        assert!((state.position - 0.0).abs() < 1e-9);
        assert!((state.velocity - 5.0).abs() < 1e-9);
        Ok(())
    }

    #[test]
    fn test_underdamped_oscillation() -> Result<(), SpringError> {
        // DEFAULT: zeta = 0.5 (< 1.0) -> Underdamped
        let spring = Spring::new(SpringConfig::DEFAULT, 0.0, 100.0, 0.0)?;
        assert_eq!(spring.regime(), DampingRegime::Underdamped);

        // At t = 0, position = 0
        assert!((spring.sample_position(Duration::ZERO) - 0.0).abs() < 1e-6);

        // Underdamped should overshoot target (100.0) initially
        let mut max_pos = 0.0;
        for i in 1..=50 {
            let t = Duration::from_secs_f64(i as f64 * 0.02);
            let pos = spring.sample_position(t);
            if pos > max_pos {
                max_pos = pos;
            }
        }
        assert!(max_pos > 100.0, "Underdamped spring should overshoot target 100.0 (got max {max_pos})");
        Ok(())
    }

    #[test]
    fn test_critically_damped() -> Result<(), SpringError> {
        // For mass = 1.0, stiffness = 100.0, w0 = 10.0. Critical damping c = 2 * m * w0 = 20.0.
        let config = SpringConfig::new(1.0, 100.0, 20.0)?;
        let spring = Spring::new(config, 0.0, 100.0, 0.0)?;
        assert_eq!(spring.regime(), DampingRegime::CriticallyDamped);

        // Monotonic convergence without overshoot
        let mut prev_pos = 0.0;
        for i in 1..=100 {
            let t = Duration::from_secs_f64(i as f64 * 0.01);
            let pos = spring.sample_position(t);
            assert!(pos >= prev_pos, "Critically damped spring should not oscillate/decrease once moving forward");
            assert!(pos <= 100.0 + 1e-5, "Critically damped spring should not overshoot");
            prev_pos = pos;
        }
        Ok(())
    }

    #[test]
    fn test_overdamped() -> Result<(), SpringError> {
        // High damping: c = 50.0 > 20.0 -> Overdamped
        let config = SpringConfig::new(1.0, 100.0, 50.0)?;
        let spring = Spring::new(config, 0.0, 100.0, 0.0)?;
        assert_eq!(spring.regime(), DampingRegime::Overdamped);

        let mut prev_pos = 0.0;
        for i in 1..=100 {
            let t = Duration::from_secs_f64(i as f64 * 0.02);
            let pos = spring.sample_position(t);
            assert!(pos >= prev_pos);
            assert!(pos <= 100.0 + 1e-5);
            prev_pos = pos;
        }
        Ok(())
    }

    #[test]
    fn test_near_critical_boundary() -> Result<(), SpringError> {
        // zeta = 1.0 + 1e-6 (within 1e-5 boundary epsilon)
        let w0 = 10.0;
        let m = 1.0;
        let k = 100.0;
        let zeta = 1.0 + 1e-6;
        let c = 2.0 * m * w0 * zeta;
        let config = SpringConfig::new(m, k, c)?;
        let spring = Spring::new(config, 0.0, 100.0, 0.0)?;
        assert_eq!(spring.regime(), DampingRegime::CriticallyDamped);
        Ok(())
    }

    #[test]
    fn test_retarget_pure_functional() -> Result<(), SpringError> {
        let spring = Spring::new(SpringConfig::DEFAULT, 0.0, 100.0, 0.0)?;
        let t = Duration::from_millis(150);
        let new_spring = spring.retargeted(250.0, t);

        // Original spring should be unchanged
        assert_eq!(spring.target_position(), 100.0);
        assert_eq!(spring.initial_position(), 0.0);

        // New spring should have target 250.0 and initial state = spring.sample(t)
        let state_at_t = spring.sample(t);
        assert_eq!(new_spring.target_position(), 250.0);
        assert!((new_spring.initial_position() - state_at_t.position).abs() < 1e-9);
        assert!((new_spring.initial_velocity() - state_at_t.velocity).abs() < 1e-9);
        Ok(())
    }

    #[test]
    fn test_retarget_c1_continuity() -> Result<(), SpringError> {
        let configs = [
            SpringConfig::DEFAULT,
            SpringConfig::GENTLE,
            SpringConfig::WOBBLY,
            SpringConfig::STIFF,
            SpringConfig::BOUNCY,
            SpringConfig::new(1.0, 100.0, 20.0)?, // Critically damped
            SpringConfig::new(1.0, 100.0, 50.0)?, // Overdamped
        ];

        for config in configs {
            let spring = Spring::new(config, 0.0, 100.0, 10.0)?;
            let t_retarget = Duration::from_millis(123);
            let old_state = spring.sample(t_retarget);

            let new_spring = spring.retargeted(300.0, t_retarget);
            let new_state_at_zero = new_spring.sample(Duration::ZERO);

            assert!((new_state_at_zero.position - old_state.position).abs() < 1e-9);
            assert!((new_state_at_zero.velocity - old_state.velocity).abs() < 1e-9);
        }
        Ok(())
    }

    #[test]
    fn test_consecutive_retargeting() -> Result<(), SpringError> {
        let mut spring = Spring::new(SpringConfig::DEFAULT, 0.0, 100.0, 0.0)?;
        
        let targets = [50.0, 200.0, 0.0, 150.0];
        let mut current_t = Duration::ZERO;

        for &target in &targets {
            current_t += Duration::from_millis(30);
            let state_before = spring.sample(current_t);
            spring.retarget(target, current_t);

            assert_eq!(spring.target_position(), target);
            assert!((spring.initial_position() - state_before.position).abs() < 1e-9);
            assert!((spring.initial_velocity() - state_before.velocity).abs() < 1e-9);

            let state_after_retarget = spring.sample(Duration::ZERO);
            assert!((state_after_retarget.position - state_before.position).abs() < 1e-9);
            assert!((state_after_retarget.velocity - state_before.velocity).abs() < 1e-9);
        }

        Ok(())
    }

    #[test]
    fn test_settle_detection() -> Result<(), SpringError> {
        let spring = Spring::new(SpringConfig::DEFAULT, 0.0, 100.0, 0.0)?;
        // At t = 0, position = 0, target = 100 -> not settled
        assert!(!spring.is_settled(Duration::ZERO));
        assert!(!spring.settled(Duration::ZERO));

        // Mid flight -> not settled
        assert!(!spring.is_settled(Duration::from_millis(50)));

        // After long time (e.g. 5 seconds) -> settled
        let t_settled = Duration::from_secs(5);
        assert!(spring.is_settled(t_settled));
        assert!(spring.settled(t_settled));

        // Custom thresholds
        assert!(spring.is_settled_with_thresholds(t_settled, 0.001, 0.001));
        Ok(())
    }

    #[test]
    fn test_settle_duration_estimation() -> Result<(), SpringError> {
        // Underdamped
        let spring_under = Spring::new(SpringConfig::DEFAULT, 0.0, 100.0, 0.0)?;
        let dur_under = spring_under.settle_duration();
        assert!(dur_under.is_some());
        let d_under = dur_under.unwrap();
        assert!(spring_under.is_settled(d_under));

        // Critically Damped
        let cfg_crit = SpringConfig::new(1.0, 100.0, 20.0)?;
        let spring_crit = Spring::new(cfg_crit, 0.0, 100.0, 0.0)?;
        let dur_crit = spring_crit.settle_duration();
        assert!(dur_crit.is_some());
        assert!(spring_crit.is_settled(dur_crit.unwrap()));

        // Overdamped
        let cfg_over = SpringConfig::new(1.0, 100.0, 50.0)?;
        let spring_over = Spring::new(cfg_over, 0.0, 100.0, 0.0)?;
        let dur_over = spring_over.settle_duration();
        assert!(dur_over.is_some());
        assert!(spring_over.is_settled(dur_over.unwrap()));

        // Undamped (damping = 0.0) -> None
        let cfg_undamped = SpringConfig::new(1.0, 100.0, 0.0)?;
        let spring_undamped = Spring::new(cfg_undamped, 0.0, 100.0, 0.0)?;
        assert!(spring_undamped.settle_duration().is_none());

        Ok(())
    }
}


