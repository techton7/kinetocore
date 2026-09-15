<p align="center">
  <img src="https://raw.githubusercontent.com/techton7/kinetocore/main/assets/icon.svg" alt="kinetocore logo" width="160" height="160" />
</p>

<h1 align="center">kinetocore</h1>

<p align="center">
  <strong>Pure Rust, framework-agnostic physics and multi-track timeline motion engine.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/kinetocore"><img src="https://img.shields.io/crates/v/kinetocore.svg" alt="Crates.io" /></a>
  <a href="https://docs.rs/kinetocore"><img src="https://docs.rs/kinetocore/badge.svg" alt="docs.rs" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg" alt="License" /></a>
</p>

---

**Kinetocore** is a high-performance, headless motion and multi-track timeline orchestration engine written in 100% pure Rust.

Designed with **zero UI or windowing dependencies**, `kinetocore` provides nanosecond-precision numerical interpolation, velocity-based spring physics, and time-scrubbable sequencers. It serves as the foundational mathematical engine behind [**Kinetoxus**](https://github.com/techton7/kinetoxus) and can be embedded in any Rust application (WGPU, Bevy, Slint, egui, or headless simulation).

## 🧬 Biological Etymology

Named after the **Kinetochore**, the cellular protein disc that attaches chromosomes to spindle fibers and generates physical tension to drive chromosome segregation during cell division. Kinetocore is the underlying engine generating physical motion forces across time and space.

---

## ⚡ Core Architectural Pillars

### 1. Zero UI Dependencies (Headless & Deterministic)
- Completely decoupled from any rendering framework or GUI event loop.
- 100% deterministically testable in unit tests without window handles, display servers, or mocks.

### 2. Dual Mathematics Engine
- **Robert Penner Easing Formulas** (via [`easer`]): Time-tested mathematical curves (Quadratic, Cubic, Exponential, Elastic, Bounce).
- **Damped Harmonic Oscillator** (Spring Physics): Velocity-based spring mechanics ($F = -kx - cv$) supporting real-time user gesture interrupts, velocity inheritance, and bounce settling.

### 3. Multi-Track Timeline & First-Class Scrubbing
- Multi-track time sequencer with keyframes, delays, staggers, and callbacks (`on_update`, `on_complete`).
- `.seek(seconds: f64)`: Instant jump to any timestamp.
- `.progress(ratio: f64)`: Scrub from 0.0 to 1.0 (ideal for sliders, scrollbars, or video players).
- `.reverse()`, `.time_scale(rate: f64)`: Smooth playback reversal and speed scaling.

### 4. Target-Agnostic Numeric Interpolation
Interpolates any data type implementing the interpolation contract:
- Scalars: `f32`, `f64`
- Arrays & Vectors: `[f32; N]`, `Vec2`, `Vec3`, `Vec4`
- Direct WGPU Buffers: Write directly into GPU uniform and instance buffers (`queue.write_buffer`) with zero VDOM overhead.

---

## 🗺️ Ecosystem Role

```text
┌─────────────────────────────────────────────────────────────┐
│ kinetocore (Universal Core - Zero UI Dependency)            │
│ - Robert Penner Easing & Damped Harmonic Oscillator         │
│ - Multi-Track Timeline Sequencer (seek, progress, reverse)  │
│ - Target-Agnostic [f32; N] & Scalar Interpolation           │
└──────────────────────────────┬──────────────────────────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         ▼                     ▼                     ▼
[ kinetoxus (Dioxus) ]   [ trioxus (WGPU) ]   [ nodoxus (Graph) ]
- use_timeline() hooks   - 3D camera orbits   - Layout sliding
- Signal synchronization - Shader uniforms    - Edge pulse flows
- RSX CSS style bindings - Direct GPU buffers - FitView gliding
```

---

## 💻 Quick Code Preview

```rust
use std::time::Duration;
use kinetocore::prelude::*;

// 1. Create a scrubbable multi-track timeline
let mut timeline = Timeline::new();

// 2. Schedule tracks
let mut camera_pos = [0.0f32, 0.0, 10.0];
timeline.track(&mut camera_pos)
    .to([15.0, 5.0, 25.0], Duration::from_secs_f32(2.0))
    .ease(Ease::CubicOut);

// 3. Advance time (e.g. inside a 120fps render loop)
timeline.tick(Duration::from_millis(16));

// 4. Or scrub directly to 50% progress
timeline.progress(0.5);
```

---

## 🗺️ Development Milestones

- [x] **Project Initialization**: Repository scaffold, dual MIT/Apache-2.0 licenses, Release-plz CI setup.
- [ ] **M1 (Math & Interpolation Core)**: Penner easing integration, numeric interpolation trait, and spring physics models.
- [ ] **M2 (Tween & Timeline Engine)**: Multi-track sequencer, keyframe scheduler, `play()`, `reverse()`, and `seek()` controls.
- [ ] **M3 (Zero-Copy Buffer Adapter)**: Direct slice writing helpers for WGPU buffers.

---

## 📜 License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
