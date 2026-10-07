# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-10-08

### Added

- Closed-form analytical spring solver supporting underdamped, critically damped, and overdamped regimes (`kinetocore::spring`).
- Near-critical stability boundary protection ($|\zeta - 1.0| \le 10^{-5}$) eliminating division-by-zero or numerical blowup.
- Pure $C^1$ velocity-preserving mid-flight retargeting.
- Settle detection (`is_settled`) and settle duration estimation.
- Comprehensive analytical verification test suite with sampling cadence independence.

## [0.1.1](https://github.com/techton7/kinetocore/compare/v0.1.0...v0.1.1) - 2026-09-21

### Added

- *(kinetocore)* add phase-1 value core

### Other

- release v0.1.0

## [0.1.0](https://github.com/techton7/kinetocore/releases/tag/v0.1.0) - 2026-09-15

### Added

- initial commit for kinetocore
