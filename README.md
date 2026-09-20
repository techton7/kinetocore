<p align="center">
  <img src="https://raw.githubusercontent.com/techton7/kinetocore/main/assets/icon.svg" alt="kinetocore logo" width="160" height="160" />
</p>

<h1 align="center">kinetocore</h1>

<p align="center">
  <strong>Pure Rust, framework-agnostic value-first tweening and motion core.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/kinetocore"><img src="https://img.shields.io/crates/v/kinetocore.svg" alt="Crates.io" /></a>
  <a href="https://docs.rs/kinetocore"><img src="https://docs.rs/kinetocore/badge.svg" alt="docs.rs" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg" alt="License" /></a>
</p>

---

**Kinetocore** is a high-performance, headless motion core written in 100% pure Rust.

Designed with **zero UI or windowing dependencies**, `kinetocore` currently provides a deterministic value-first tweening foundation: interpolation contracts, easing curves, playback clocks, repeat/yoyo behavior, and pure-value `set` / `from_to` tween evaluation. It serves as the foundational mathematical engine behind [**Kinetoxus**](https://github.com/techton7/kinetoxus) and is intended to remain reusable across Dioxus, graphics runtimes, and other non-UI hosts.

## 🧬 Biological Etymology

Named after the **Kinetochore**, the cellular protein disc that attaches chromosomes to spindle fibers and generates physical tension to drive chromosome segregation during cell division. Kinetocore is the underlying engine generating physical motion forces across time and space.

---

## ⚡ Core Architectural Pillars

### 1. Zero UI Dependencies (Headless & Deterministic)
- Completely decoupled from any rendering framework or GUI event loop.
- 100% deterministically testable in unit tests without window handles, display servers, or mocks.

### 2. Value-First Motion Core
- **`Interpolate`**: Atomic linear interpolation contract for value types.
- **`Ease`**: Unified enum wrapping Robert Penner easing formulas via [`easer`].
- **`AnimClock`**: Headless playback timekeeper with repeat and mirrored/yoyo arithmetic.
- **`Tween<T>`**: Pure-value tweening supporting deterministic `set` and `from_to`.

### 3. Target-Agnostic Numeric Interpolation
Interpolates any data type implementing the interpolation contract:
- Scalars: `f32`, `f64`
- Fixed arrays: `[f32; 2]`, `[f32; 3]`, `[f32; 4]`

### 4. Planned Later Layers
The following are intentionally **not** part of the current phase-1 surface yet:

- `to` / `from` verbs requiring current-value sampling
- target/lens abstractions for in-place host mutation
- multi-track timelines and sequencing
- spring physics
- overwrite/conflict policy
- higher-level callback orchestration

---

## 🗺️ Ecosystem Role

```text
┌─────────────────────────────────────────────────────────────┐
│ kinetocore (Universal Core - Zero UI Dependency)            │
│ - Interpolate / Ease / AnimClock / Tween                    │
│ - Deterministic set + from_to evaluation                    │
│ - Headless value-first motion foundation                    │
└──────────────────────────────┬──────────────────────────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         ▼                     ▼                     ▼
[ kinetoxus (Dioxus) ]   [ trioxus (WGPU) ]   [ nodoxus (Graph) ]
- SignalTarget adapters   - Camera/handle motion - Layout sliding
- Dioxus motion hooks     - Buffer-facing motion - Edge pulse flows
- UI-facing ergonomics    - Non-signal targets   - FitView gliding
```

---

## 💻 Quick Code Preview

```rust
use std::time::Duration;
use kinetocore::prelude::*;

// 1. Create a deterministic tween from A to B
let mut tween = Tween::from_to(
    [0.0f32, 0.0, 10.0],
    [15.0f32, 5.0, 25.0],
    Duration::from_secs_f32(2.0),
)
.ease(Ease::CubicOut)
.repeat(2)
.yoyo(true);

// 2. Advance time
let (value, state) = tween.step(Duration::from_millis(16));

// 3. Scrub directly if needed
let halfway = tween.seek(Duration::from_secs(1));

assert_eq!(halfway.len(), 3);
println!("{value:?} / {state:?}");
```

---

## 🗺️ Development Milestones

- [x] **Project Initialization**: Repository scaffold, dual MIT/Apache-2.0 licenses, Release-plz CI setup.
- [x] **M1 (Value Core Foundation)**: `Interpolate`, `Ease`, `AnimClock`, repeat/yoyo arithmetic, and deterministic `set` / `from_to`.
- [ ] **M2 (Target-Aware Tween Expansion)**: `to`, `from`, lazy init/current-value sampling, and target/lens abstraction.
- [ ] **M3 (Timeline & Sequence Engine)**: Multi-track sequencing, orchestration, and richer playback control.
- [ ] **M4 (Spring & Higher-Level Motion)**: Spring physics, overwrite policy, and broader motion semantics.

---

## 📜 License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
