//! Boundary evaluation policies and type-erased track sampling contracts.

use std::time::Duration;

/// Deterministic boundary evaluation policy for timeline tracks.
///
/// Dictates playback and evaluation semantics when querying timestamps
/// that fall outside of active clip intervals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryPolicy {
    /// Pre-roll policy ($t < t_{\text{start}}$): holds initial value of the track.
    HoldInitial,
    /// Inter-clip gap policy ($t_{\text{prev.end}} < t < t_{\text{next.start}}$): forward-fills terminal value of preceding clip.
    ForwardFill,
    /// Post-roll policy ($t > t_{\text{end}}$): holds terminal value of the track's last clip.
    HoldTerminal,
}

/// Type-erased trait for querying and sampling compiled tracks without knowing their target generic type.
pub trait TrackSampler: std::any::Any + Send + Sync {
    /// Return the name of the track.
    fn name(&self) -> &str;
    /// Return the total duration of the track.
    fn duration(&self) -> Duration;
    /// Cast this track to `&dyn Any` for type-erased downcasting.
    fn as_any(&self) -> &dyn std::any::Any;
}
