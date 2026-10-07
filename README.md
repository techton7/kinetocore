# Kinetocore

Pure Rust headless animation and spring engine providing core primitives for interpolation, easing, clocks, tweens, and analytical spring physics.

## Features

- **Interpolation & Easing**: Generic numeric/array interpolation with standard easing curves.
- **Clocks & Tweens**: Precise time control, repeat, yoyo, seek, and reverse semantics.
- **Analytical Spring Engine**: Closed-form analytical spring solver (`Spring`, `SpringConfig`) supporting:
  - Presets: `DEFAULT`, `GENTLE`, `WOBBLY`, `STIFF`, `BOUNCY`.
  - Three damping regimes: Underdamped, Critically Damped, Overdamped, plus near-critical stability guard.
  - Zero numerical drift via exact closed-form solutions for $x(t)$ and $v(t)$.
  - Pure $C^1$ continuous mid-flight retargeting (`retarget`).
  - Robust settle detection (`is_settled`) and settle duration estimation.

## Development Milestones

- [x] M1: Core Interpolation & Easing
- [x] M2: Clocks & Tweens
- [x] M3: Target-Aware Value Wrappers (`SignalTarget`, `HandleTarget`)
- [x] M4: Headless Analytical Spring Engine & Retargeting Semantics
