# Specification: Spring Physics Kernel (`kinetocore`)

## 1. Physical Model & Parameter Formulation

The spring kernel models a classic 1D mass-spring-damper system governed by Hooke's Law and viscous damping:

$$m \frac{d^2 x}{dt^2} + c \frac{dx}{dt} + k (x - x_{target}) = 0$$

### Parameters
- **Mass ($m > 0$)**: Inertia of the moving body. Must be strictly positive ($m > 0$).
- **Stiffness ($k > 0$)**: Spring constant determining restoring force intensity. Must be strictly positive ($k > 0$).
- **Damping ($c \ge 0$)**: Friction/damping coefficient opposing velocity. Must be non-negative ($c \ge 0$).
- **Initial Position ($x_0$)**: Starting position at $t = 0$.
- **Target Position ($x_{target}$)**: Equilibrium destination position.
- **Initial Velocity ($v_0$)**: Starting velocity at $t = 0$.

### Derived Parameters
- **Natural Angular Frequency ($\omega_0$)**:
  $$\omega_0 = \sqrt{\frac{k}{m}}$$
- **Damping Ratio ($\zeta$)**:
  $$\zeta = \frac{c}{2 \sqrt{m k}} = \frac{c}{2 m \omega_0}$$

### Parameter Validation Rules & Error Conditions
1. $m \le 0$: Return explicit error `SpringError::InvalidMass` (mass must be positive).
2. $k \le 0$: Return explicit error `SpringError::InvalidStiffness` (stiffness must be positive).
3. $c < 0$: Return explicit error `SpringError::InvalidDamping` (damping must be non-negative).

---

## 2. Analytical Closed-Form Solutions across 3 Damping Regimes

Let relative displacement be $y(t) = x(t) - x_{target}$.
Initial conditions for $y(t)$:
- $y(0) = y_0 = x_0 - x_{target}$
- $y'(0) = v(0) = v_0$

### A. Underdamped Regime ($\zeta < 1$)
The system oscillates with decaying amplitude.
- Damped angular frequency:
  $$\omega_d = \omega_0 \sqrt{1 - \zeta^2}$$
- Relative displacement $y(t)$:
  $$y(t) = e^{-\zeta \omega_0 t} \left( y_0 \cos(\omega_d t) + \frac{v_0 + \zeta \omega_0 y_0}{\omega_d} \sin(\omega_d t) \right)$$
- Velocity $v(t) = y'(t)$:
  $$v(t) = -e^{-\zeta \omega_0 t} \left[ \left( \zeta \omega_0 y_0 - (v_0 + \zeta \omega_0 y_0) \right) \frac{\sin(\omega_d t)}{\omega_d} + \ldots \right]$$
  More precisely:
  $$v(t) = e^{-\zeta \omega_0 t} \left( v_0 \cos(\omega_d t) - \frac{\omega_0^2 y_0 + \zeta \omega_0 v_0}{\omega_d} \sin(\omega_d t) \right)$$

### B. Critically Damped Regime ($\zeta = 1$)
The system returns to equilibrium as fast as possible without oscillation.
- Relative displacement $y(t)$:
  $$y(t) = e^{-\omega_0 t} \left( y_0 + (v_0 + \omega_0 y_0) t \right)$$
- Velocity $v(t)$:
  $$v(t) = e^{-\omega_0 t} \left( v_0 - \omega_0 (v_0 + \omega_0 y_0) t \right)$$

### C. Overdamped Regime ($\zeta > 1$)
The system returns to equilibrium sluggishly without oscillating.
- Characteristic roots:
  $$r_1 = -\zeta \omega_0 + \omega_0 \sqrt{\zeta^2 - 1}$$
  $$r_2 = -\zeta \omega_0 - \omega_0 \sqrt{\zeta^2 - 1}$$
- Relative displacement $y(t)$:
  $$y(t) = e^{-\zeta \omega_0 t} \left( C_1 \cosh(\omega_d' t) + C_2 \sinh(\omega_d' t) \right)$$
  where $\omega_d' = \omega_0 \sqrt{\zeta^2 - 1}$, and:
  $$C_1 = y_0$$
  $$C_2 = \frac{v_0 + \zeta \omega_0 y_0}{\omega_d'}$$
- Velocity $v(t)$:
  $$v(t) = e^{-\zeta \omega_0 t} \left( (C_2 \omega_d' - \zeta \omega_0 C_1) \cosh(\omega_d' t) + (C_1 \omega_d' - \zeta \omega_0 C_2) \sinh(\omega_d' t) \right)$$

### D. Near-Critical Boundary Stability
To prevent numerical instability, division by near-zero, or catastrophic cancellation when $\zeta$ is extremely close to $1.0$:
- Define boundary epsilon: $\epsilon_{boundary} = 10^{-5}$.
- If $|\zeta - 1.0| < \epsilon_{boundary}$, smoothly fall back or clamp to the **Critically Damped ($\zeta = 1$ )** analytical formula.

---

## 3. Settle Detection & Derived Duration Semantics

### Settle Condition
A spring state is considered settled when both position error and velocity fall within tight thresholds:
$$|x(t) - x_{target}| \le \epsilon_x \quad \text{AND} \quad |v(t)| \le \epsilon_v$$
- Default thresholds:
  - $\epsilon_x = 0.001$
  - $\epsilon_v = 0.001$

### Settle Duration Semantics
- The estimated settle duration is a **derived metadata metric** computed analytically or estimated from damping envelopes.
- It is **non-authoritative** and does not replace continuous frame-by-frame simulation sampling.

---

## 4. Pure State Retargeting (Interruption)

To support real-time user interaction and animation interruptions without jarring discontinuities:
1. At interruption timestamp $t_i$, sample current instantaneous position $x(t_i)$ and velocity $v(t_i)$ from the active spring state.
2. Construct a fresh spring state with:
   - Initial position $x_0' = x(t_i)$
   - Initial velocity $v_0' = v(t_i)$
   - Target position $x_{target}' = x_{new}$
3. **Physical Continuity**: This guarantees strict $C^1$ continuity (both position $x$ and velocity $v$ are continuous across the retargeting boundary). No mutable history loops or abrupt velocity spikes occur.

---

## 5. Phase Scope Locks

### In Scope
- Pure 1D scalar analytical spring kernel (`kinetocore`).
- Exact closed-form solutions for underdamped, critically damped, and overdamped regimes.
- Near-critical boundary stabilization.
- Settle detection and derived duration metrics.
- Pure state retargeting ensuring $C^1$ continuity.
- Comprehensive deterministic unit tests.

### Explicitly Out of Scope
- Timelines and complex multi-keyframe tracks.
- Position parameter arrays and multi-dimensional coordinate spaces.
- Dioxus framework integration or hooks (`use_spring`).
- Signal / reactive handle target adapters.
- UI runtimes, rendering loops, or browser bindings.

---

## 6. Validation

- Verification command:
  ```bash
  bash ~/.agents/skills/verify-markdown/bin/verify-markdown.sh /Volumes/HDD-1T-2021-Mac/Vault/business/project/mine/dioxus/animation/kinetocore/SPEC_SPRING.md --require-frontmatter false
  ```
- Must pass with exit code `0`.
