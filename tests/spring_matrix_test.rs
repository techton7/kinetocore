//! Headless mathematical verification matrix for `kinetocore::spring::Spring`.
//!
//! Verifies AC 1 (analytical precision against theoretical curves), AC 2 (near-critical
//! boundary stability), AC 3 (velocity & retarget continuity), and AC 4 (sampling cadence
//! independence & 100% pass rate).

use kinetocore::spring::{Spring, SpringConfig};

#[test]
fn test_ac1_analytical_precision_curves() {
    // 1. Underdamped (m = 1.0, k = 100.0, c = 10.0 => omega_0 = 10, zeta = 0.5, wd = 10 * sqrt(0.75) ~ 8.660254037844386)
    let config_under = SpringConfig::new(1.0, 100.0, 10.0).unwrap();
    let y0_val = 1.0; // initial = 1.0, target = 0.0 => y0 = 1.0
    let v0_val = 0.0;
    let spring_under = Spring::new(config_under, y0_val, 0.0, v0_val).unwrap();

    let omega_0 = 10.0f64;
    let zeta = 0.5f64;
    let wd = omega_0 * (1.0 - zeta * zeta).sqrt();

    let sample_times = [0.05, 0.1, 0.2, 0.5];
    for &t in &sample_times {
        let state = spring_under.sample_secs(t);
        // Theoretical formulas
        let e_term = (-zeta * omega_0 * t).exp();
        let cos_term = (wd * t).cos();
        let sin_term = (wd * t).sin();

        let y_theo = e_term * (y0_val * cos_term + ((v0_val + zeta * omega_0 * y0_val) / wd) * sin_term);
        let v_theo = e_term * (v0_val * cos_term - ((omega_0 * omega_0 * y0_val + zeta * omega_0 * v0_val) / wd) * sin_term);

        assert!(
            (state.position - y_theo).abs() <= 1e-5,
            "Underdamped position mismatch at t = {t}: got {}, expected {}",
            state.position, y_theo
        );
        assert!(
            (state.velocity - v_theo).abs() <= 1e-5,
            "Underdamped velocity mismatch at t = {t}: got {}, expected {}",
            state.velocity, v_theo
        );
    }

    // 2. Critically Damped (m = 1.0, k = 100.0, c = 20.0 => omega_0 = 10, zeta = 1.0)
    let config_crit = SpringConfig::new(1.0, 100.0, 20.0).unwrap();
    let spring_crit = Spring::new(config_crit, y0_val, 0.0, v0_val).unwrap();

    for &t in &sample_times {
        let state = spring_crit.sample_secs(t);
        let e_term = (-10.0 * t).exp();
        let y_theo = e_term * (y0_val + (v0_val + 10.0 * y0_val) * t);
        let v_theo = e_term * (v0_val - 10.0 * (v0_val + 10.0 * y0_val) * t);

        assert!(
            (state.position - y_theo).abs() <= 1e-5,
            "Critically damped position mismatch at t = {t}: got {}, expected {}",
            state.position, y_theo
        );
        assert!(
            (state.velocity - v_theo).abs() <= 1e-5,
            "Critically damped velocity mismatch at t = {t}: got {}, expected {}",
            state.velocity, v_theo
        );
    }

    // 3. Overdamped (m = 1.0, k = 100.0, c = 25.0 => zeta = 1.25, wd_prime = 10 * sqrt(1.25^2 - 1) = 10 * sqrt(0.5625) = 7.5)
    let config_over = SpringConfig::new(1.0, 100.0, 25.0).unwrap();
    let spring_over = Spring::new(config_over, y0_val, 0.0, v0_val).unwrap();

    let zeta_over = 1.25f64;
    let wd_prime = 7.5f64;
    for &t in &sample_times {
        let state = spring_over.sample_secs(t);
        let e_term = (-zeta_over * omega_0 * t).exp();
        let c1 = y0_val;
        let c2 = (v0_val + zeta_over * omega_0 * y0_val) / wd_prime;
        let cosh_term = (wd_prime * t).cosh();
        let sinh_term = (wd_prime * t).sinh();

        let y_theo = e_term * (c1 * cosh_term + c2 * sinh_term);
        let v_theo = e_term * ((c2 * wd_prime - zeta_over * omega_0 * c1) * cosh_term + (c1 * wd_prime - zeta_over * omega_0 * c2) * sinh_term);

        assert!(
            (state.position - y_theo).abs() <= 1e-5,
            "Overdamped position mismatch at t = {t}: got {}, expected {}",
            state.position, y_theo
        );
        assert!(
            (state.velocity - v_theo).abs() <= 1e-5,
            "Overdamped velocity mismatch at t = {t}: got {}, expected {}",
            state.velocity, v_theo
        );
    }
}

#[test]
fn test_ac2_near_critical_boundary_stability() {
    let m = 1.0;
    let k = 100.0;
    let omega_0 = 10.0;

    let zetas = [
        1.0 - 1e-6,             // inside near-critical boundary
        1.0,                    // exact critical damping
        1.0 + 1e-6,             // inside near-critical boundary
        1.0 - 2.0 * 1e-5,       // just outside boundary, underdamped
        1.0 + 2.0 * 1e-5,       // just outside boundary, overdamped
    ];

    let y0 = 1.0;
    let v0 = 0.0;
    let sample_times = [0.01, 0.05, 0.1, 0.2, 0.5, 1.0];

    let mut springs = Vec::new();
    for &zeta in &zetas {
        let c = 2.0 * m * omega_0 * zeta;
        let config = SpringConfig::new(m, k, c).unwrap();
        let spring = Spring::new(config, y0, 0.0, v0).unwrap();
        springs.push((zeta, spring));
    }

    for &t in &sample_times {
        let mut positions = Vec::new();
        let mut velocities = Vec::new();

        for &(zeta, ref spring) in &springs {
            let state = spring.sample_secs(t);
            assert!(state.position.is_finite(), "Position is not finite for zeta = {zeta} at t = {t}");
            assert!(state.velocity.is_finite(), "Velocity is not finite for zeta = {zeta} at t = {t}");
            positions.push(state.position);
            velocities.push(state.velocity);
        }

        let base_p = positions[1]; // exact critical damping as reference
        let base_v = velocities[1];

        for (i, &(zeta, _)) in springs.iter().enumerate() {
            let dp = (positions[i] - base_p).abs();
            let dv = (velocities[i] - base_v).abs();
            assert!(
                dp < 1e-4,
                "Position divergence {dp} exceeds 1e-4 at t = {t} for zeta = {zeta}"
            );
            assert!(
                dv < 1e-4,
                "Velocity divergence {dv} exceeds 1e-4 at t = {t} for zeta = {zeta}"
            );
        }
    }
}

#[test]
fn test_ac3_velocities_and_retarget_continuity() {
    let config = SpringConfig::default(); // m=1, k=100, c=10

    let v0_cases = [0.0, 5.0, -5.0];
    for &v0 in &v0_cases {
        let spring = Spring::new(config, 0.0, 10.0, v0).unwrap();
        let state_0 = spring.sample_secs(0.0);
        assert_eq!(state_0.position, 0.0);
        assert_eq!(state_0.velocity, v0);

        let state_eps = spring.sample_secs(1e-4);
        assert!(state_eps.position.is_finite());
        assert!(state_eps.velocity.is_finite());
    }

    let spring = Spring::new(config, 0.0, 10.0, 2.0).unwrap();
    let t_trans = 0.15;
    let state_before = spring.sample_secs(t_trans);

    let new_target = 20.0;
    let retargeted = spring.retargeted_secs(new_target, t_trans);

    let state_after_start = retargeted.sample_secs(0.0);
    assert_eq!(
        state_after_start.position, state_before.position,
        "Retarget position discontinuity"
    );
    assert_eq!(
        state_after_start.velocity, state_before.velocity,
        "Retarget velocity discontinuity"
    );

    // Verify continuous acceleration / smooth velocity right after transition.
    // Note: spring.sample_secs(t_trans + dt) uses the old target (10.0), whereas
    // retargeted.sample_secs(dt) uses the new target (20.0). Since target acceleration
    // depends on position (F = -k(x - target) - c*v), changing target changes the acceleration
    // instantly, but velocity itself (state.velocity) is continuous at t_transition (dt = 0).
    // As dt -> 0+, both velocities approach state_before.velocity.
    let dt = 1e-6;
    let v_retarget_next = retargeted.sample_secs(dt).velocity;
    assert!(
        (v_retarget_next - state_before.velocity).abs() < 0.01,
        "Velocity continuity at retarget boundary: initial state v = {}, immediate post-retarget v = {}",
        state_before.velocity, v_retarget_next
    );
}

#[test]
fn test_ac4_sampling_cadence_independence() {
    let config = SpringConfig::default();
    let spring = Spring::new(config, 0.0, 10.0, 3.0).unwrap();
    let target_t = 0.5;

    let direct_state = spring.sample_secs(target_t);

    // 60 FPS stepping
    let dt_60fps = 1.0 / 60.0;
    let mut s = spring;
    let mut t_current = 0.0;
    while t_current < target_t - 1e-9 {
        let step = (target_t - t_current).min(dt_60fps);
        let st = s.sample_secs(step);
        s = Spring::new(config, st.position, 10.0, st.velocity).unwrap();
        t_current += step;
    }
    let stepped_60fps_state = s.sample_secs(0.0);

    // 120 FPS stepping
    let dt_120fps = 1.0 / 120.0;
    let mut s_120 = spring;
    let mut t_current_120 = 0.0;
    while t_current_120 < target_t - 1e-9 {
        let step = (target_t - t_current_120).min(dt_120fps);
        let st = s_120.sample_secs(step);
        s_120 = Spring::new(config, st.position, 10.0, st.velocity).unwrap();
        t_current_120 += step;
    }
    let stepped_120fps_state = s_120.sample_secs(0.0);

    // Randomized intervals stepping
    let random_steps = [0.03, 0.07, 0.012, 0.045, 0.088, 0.1, 0.05, 0.075, 0.03];
    let mut s_rand = spring;
    let mut t_current_rand = 0.0;
    for &step in &random_steps {
        if t_current_rand + step >= target_t {
            let final_step = target_t - t_current_rand;
            if final_step > 0.0 {
                let st = s_rand.sample_secs(final_step);
                s_rand = Spring::new(config, st.position, 10.0, st.velocity).unwrap();
            }
            break;
        } else {
            let st = s_rand.sample_secs(step);
            s_rand = Spring::new(config, st.position, 10.0, st.velocity).unwrap();
            t_current_rand += step;
        }
    }
    let stepped_rand_state = s_rand.sample_secs(0.0);

    let eps = 1e-12;

    assert!(
        (direct_state.position - stepped_60fps_state.position).abs() < eps,
        "60FPS position divergence: direct={}, stepped={}", direct_state.position, stepped_60fps_state.position
    );
    assert!(
        (direct_state.velocity - stepped_60fps_state.velocity).abs() < eps,
        "60FPS velocity divergence: direct={}, stepped={}", direct_state.velocity, stepped_60fps_state.velocity
    );

    assert!(
        (direct_state.position - stepped_120fps_state.position).abs() < eps,
        "120FPS position divergence: direct={}, stepped={}", direct_state.position, stepped_120fps_state.position
    );
    assert!(
        (direct_state.velocity - stepped_120fps_state.velocity).abs() < eps,
        "120FPS velocity divergence: direct={}, stepped={}", direct_state.velocity, stepped_120fps_state.velocity
    );

    assert!(
        (direct_state.position - stepped_rand_state.position).abs() < eps,
        "Random steps position divergence: direct={}, stepped={}", direct_state.position, stepped_rand_state.position
    );
    assert!(
        (direct_state.velocity - stepped_rand_state.velocity).abs() < eps,
        "Random steps velocity divergence: direct={}, stepped={}", direct_state.velocity, stepped_rand_state.velocity
    );
}
