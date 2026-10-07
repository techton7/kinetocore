//! Position grammar and temporal reference types for timeline authoring.

use std::time::Duration;

use crate::timeline::error::TimelineError;

/// Directional sign for duration offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetSign {
    /// Positive offset (added to base time).
    Positive,
    /// Negative offset (subtracted from base time).
    Negative,
}

/// A duration with an explicit positive or negative sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedDuration {
    /// Magnitude of the duration.
    pub duration: Duration,
    /// Sign indicating whether to add or subtract this duration.
    pub sign: OffsetSign,
}

impl SignedDuration {
    /// Zero signed duration.
    pub const ZERO: Self = Self {
        duration: Duration::ZERO,
        sign: OffsetSign::Positive,
    };

    /// Create a positive signed duration.
    #[inline]
    pub fn positive(duration: Duration) -> Self {
        Self {
            duration,
            sign: OffsetSign::Positive,
        }
    }

    /// Create a negative signed duration.
    #[inline]
    pub fn negative(duration: Duration) -> Self {
        Self {
            duration,
            sign: OffsetSign::Negative,
        }
    }

    /// Apply this signed duration to a base duration: `base ± duration`.
    ///
    /// If subtracting causes underflow below zero, the result saturates to `Duration::ZERO`.
    #[inline]
    pub fn apply_to(&self, base: Duration) -> Duration {
        match self.sign {
            OffsetSign::Positive => base + self.duration,
            OffsetSign::Negative => base.saturating_sub(self.duration),
        }
    }

    /// Return this signed duration as signed seconds (`f64`).
    #[inline]
    pub fn as_secs_f64(&self) -> f64 {
        let s = self.duration.as_secs_f64();
        match self.sign {
            OffsetSign::Positive => s,
            OffsetSign::Negative => -s,
        }
    }
}

/// Positioning anchor for placing clips and labels along the timeline.
#[derive(Debug, Clone, PartialEq)]
pub enum Position {
    /// Aligns at the current total duration of the timeline.
    End,
    /// Absolute timestamp from the start of the timeline.
    Absolute(Duration),
    /// Relative offset from the current total duration of the timeline.
    Offset(SignedDuration),
    /// Aligns at the start of the most recently inserted clip.
    RecentStart,
    /// Aligns at the end of the most recently inserted clip.
    RecentEnd,
    /// Relative offset from the start of the most recently inserted clip.
    RecentStartOffset(SignedDuration),
    /// Relative offset from the end of the most recently inserted clip.
    RecentEndOffset(SignedDuration),
    /// Aligns at the timestamp of a named label.
    Label(String),
    /// Relative offset from the timestamp of a named label.
    LabelOffset(String, SignedDuration),
}

impl Position {
    /// Create an absolute position from a `Duration`.
    #[inline]
    pub fn absolute(duration: Duration) -> Self {
        Self::Absolute(duration)
    }

    /// Create a relative offset position from the timeline end.
    #[inline]
    pub fn offset(duration: Duration, sign: OffsetSign) -> Self {
        Self::Offset(SignedDuration { duration, sign })
    }

    /// Create a named label position.
    #[inline]
    pub fn label(name: impl Into<String>) -> Self {
        Self::Label(name.into())
    }

    /// Create a named label offset position.
    #[inline]
    pub fn label_offset(name: impl Into<String>, duration: Duration, sign: OffsetSign) -> Self {
        Self::LabelOffset(name.into(), SignedDuration { duration, sign })
    }

    /// Parse a position string according to the timeline positioning grammar.
    ///
    /// # Supported Grammar
    /// - `""` -> `Position::End`
    /// - `"1.5"`, `"1.5s"`, `"500ms"` -> `Position::Absolute`
    /// - `"+=0.2s"`, `"+=200ms"` -> `Position::Offset(Positive)`
    /// - `"-=0.1s"`, `"-=100ms"` -> `Position::Offset(Negative)`
    /// - `"<"` -> `Position::RecentStart`
    /// - `">"` -> `Position::RecentEnd`
    /// - `"<+=0.2s"`, `"<-=0.1s"` -> `Position::RecentStartOffset`
    /// - `">+=0.5s"`, `">-=-0.2s"` -> `Position::RecentEndOffset`
    /// - `"intro"` -> `Position::Label("intro")`
    /// - `"intro+=0.3s"`, `"intro-=0.2s"` -> `Position::LabelOffset("intro", ...)`
    pub fn parse(s: &str) -> Result<Self, TimelineError> {
        let s = s.trim();
        if s.is_empty() {
            return Ok(Position::End);
        }

        if s == "<" {
            return Ok(Position::RecentStart);
        }
        if s == ">" {
            return Ok(Position::RecentEnd);
        }

        if let Some(rem) = s.strip_prefix('<') {
            let signed = parse_signed_duration(rem)?;
            return Ok(Position::RecentStartOffset(signed));
        }

        if let Some(rem) = s.strip_prefix('>') {
            let signed = parse_signed_duration(rem)?;
            return Ok(Position::RecentEndOffset(signed));
        }

        if s.starts_with("+=") || s.starts_with("-=") {
            let signed = parse_signed_duration(s)?;
            return Ok(Position::Offset(signed));
        }

        // Check if string contains "+=" or "-=" for label offset
        if let Some(idx) = s.find("+=") {
            let label = s[..idx].trim();
            let rem = s[idx + 2..].trim();
            let dur = parse_duration(rem)?;
            return Ok(Position::LabelOffset(
                label.to_string(),
                SignedDuration::positive(dur),
            ));
        }
        if let Some(idx) = s.find("-=") {
            let label = s[..idx].trim();
            let rem = s[idx + 2..].trim();
            let dur_clean = rem.trim_start_matches('-');
            let dur = parse_duration(dur_clean)?;
            return Ok(Position::LabelOffset(
                label.to_string(),
                SignedDuration::negative(dur),
            ));
        }

        // Check if it starts with digit or decimal point or +/- before digit
        let first_char = s.chars().next().unwrap();
        if first_char.is_ascii_digit() || first_char == '.' {
            let dur = parse_duration(s)?;
            return Ok(Position::Absolute(dur));
        }

        if (first_char == '+' || first_char == '-')
            && s.chars()
                .nth(1)
                .map(|c| c.is_ascii_digit() || c == '.')
                .unwrap_or(false)
        {
            let signed = parse_signed_duration(s)?;
            return Ok(Position::Offset(signed));
        }

        // Check for label with '+' or '-' offset, e.g. "intro+0.3s" or "intro-0.2s"
        if let Some(idx) = s.rfind('+') {
            let rem = s[idx + 1..].trim();
            if let Ok(dur) = parse_duration(rem) {
                let label = s[..idx].trim();
                return Ok(Position::LabelOffset(
                    label.to_string(),
                    SignedDuration::positive(dur),
                ));
            }
        }
        if let Some(idx) = s.rfind('-') {
            let rem = s[idx + 1..].trim();
            if let Ok(dur) = parse_duration(rem) {
                let label = s[..idx].trim();
                return Ok(Position::LabelOffset(
                    label.to_string(),
                    SignedDuration::negative(dur),
                ));
            }
        }

        // Otherwise, it's a plain label identifier
        Ok(Position::Label(s.to_string()))
    }
}

fn parse_duration(s: &str) -> Result<Duration, TimelineError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(TimelineError::InvalidPosition(
            "duration string is empty".into(),
        ));
    }

    if let Some(ms_str) = s.strip_suffix("ms") {
        let ms = ms_str.trim().parse::<f64>().map_err(|e| {
            TimelineError::InvalidPosition(format!("invalid millisecond duration '{s}': {e}"))
        })?;
        if ms < 0.0 || ms.is_nan() {
            return Err(TimelineError::InvalidPosition(format!(
                "duration cannot be negative or NaN: {s}"
            )));
        }
        Ok(Duration::from_secs_f64(ms / 1000.0))
    } else if let Some(s_str) = s.strip_suffix('s') {
        let secs = s_str.trim().parse::<f64>().map_err(|e| {
            TimelineError::InvalidPosition(format!("invalid second duration '{s}': {e}"))
        })?;
        if secs < 0.0 || secs.is_nan() {
            return Err(TimelineError::InvalidPosition(format!(
                "duration cannot be negative or NaN: {s}"
            )));
        }
        Ok(Duration::from_secs_f64(secs))
    } else {
        let secs = s.parse::<f64>().map_err(|e| {
            TimelineError::InvalidPosition(format!("invalid unitless duration '{s}': {e}"))
        })?;
        if secs < 0.0 || secs.is_nan() {
            return Err(TimelineError::InvalidPosition(format!(
                "duration cannot be negative or NaN: {s}"
            )));
        }
        Ok(Duration::from_secs_f64(secs))
    }
}

fn parse_signed_duration(s: &str) -> Result<SignedDuration, TimelineError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(TimelineError::InvalidPosition(
            "offset string is empty".into(),
        ));
    }

    if let Some(rem) = s.strip_prefix("+=") {
        let dur = parse_duration(rem)?;
        Ok(SignedDuration::positive(dur))
    } else if let Some(rem) = s.strip_prefix("-=-") {
        let dur = parse_duration(rem)?;
        Ok(SignedDuration::negative(dur))
    } else if let Some(rem) = s.strip_prefix("-=") {
        let dur_clean = rem.trim_start_matches('-');
        let dur = parse_duration(dur_clean)?;
        Ok(SignedDuration::negative(dur))
    } else if let Some(rem) = s.strip_prefix('+') {
        let dur = parse_duration(rem)?;
        Ok(SignedDuration::positive(dur))
    } else if let Some(rem) = s.strip_prefix('-') {
        let dur = parse_duration(rem)?;
        Ok(SignedDuration::negative(dur))
    } else {
        // Fallback: parse as positive duration if no sign specified
        let dur = parse_duration(s)?;
        Ok(SignedDuration::positive(dur))
    }
}

impl TryFrom<&str> for Position {
    type Error = TimelineError;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Position::parse(s)
    }
}

impl TryFrom<String> for Position {
    type Error = TimelineError;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Position::parse(&s)
    }
}

impl From<Duration> for Position {
    #[inline]
    fn from(d: Duration) -> Self {
        Position::Absolute(d)
    }
}

impl From<f64> for Position {
    #[inline]
    fn from(secs: f64) -> Self {
        Position::Absolute(if secs < 0.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(secs)
        })
    }
}

impl From<f32> for Position {
    #[inline]
    fn from(secs: f32) -> Self {
        Position::Absolute(if secs < 0.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f32(secs)
        })
    }
}

impl From<SignedDuration> for Position {
    #[inline]
    fn from(sd: SignedDuration) -> Self {
        Position::Offset(sd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert_eq!(Position::parse("").unwrap(), Position::End);
        assert_eq!(Position::parse("   ").unwrap(), Position::End);
    }

    #[test]
    fn test_parse_absolute() {
        assert_eq!(
            Position::parse("1.5").unwrap(),
            Position::Absolute(Duration::from_secs_f64(1.5))
        );
        assert_eq!(
            Position::parse("1.5s").unwrap(),
            Position::Absolute(Duration::from_secs_f64(1.5))
        );
        assert_eq!(
            Position::parse("500ms").unwrap(),
            Position::Absolute(Duration::from_millis(500))
        );
    }

    #[test]
    fn test_parse_relative_offset() {
        assert_eq!(
            Position::parse("+=0.2s").unwrap(),
            Position::Offset(SignedDuration::positive(Duration::from_millis(200)))
        );
        assert_eq!(
            Position::parse("+=200ms").unwrap(),
            Position::Offset(SignedDuration::positive(Duration::from_millis(200)))
        );
        assert_eq!(
            Position::parse("-=0.1s").unwrap(),
            Position::Offset(SignedDuration::negative(Duration::from_millis(100)))
        );
        assert_eq!(
            Position::parse("-=100ms").unwrap(),
            Position::Offset(SignedDuration::negative(Duration::from_millis(100)))
        );
    }

    #[test]
    fn test_parse_recent_clips() {
        assert_eq!(Position::parse("<").unwrap(), Position::RecentStart);
        assert_eq!(Position::parse(">").unwrap(), Position::RecentEnd);
        assert_eq!(
            Position::parse("<+=0.2s").unwrap(),
            Position::RecentStartOffset(SignedDuration::positive(Duration::from_millis(200)))
        );
        assert_eq!(
            Position::parse("<-=0.1s").unwrap(),
            Position::RecentStartOffset(SignedDuration::negative(Duration::from_millis(100)))
        );
        assert_eq!(
            Position::parse(">+=0.5s").unwrap(),
            Position::RecentEndOffset(SignedDuration::positive(Duration::from_millis(500)))
        );
        assert_eq!(
            Position::parse(">-=-0.2s").unwrap(),
            Position::RecentEndOffset(SignedDuration::negative(Duration::from_millis(200)))
        );
    }

    #[test]
    fn test_parse_labels() {
        assert_eq!(
            Position::parse("intro").unwrap(),
            Position::Label("intro".to_string())
        );
        assert_eq!(
            Position::parse("intro+=0.3s").unwrap(),
            Position::LabelOffset(
                "intro".to_string(),
                SignedDuration::positive(Duration::from_millis(300))
            )
        );
        assert_eq!(
            Position::parse("intro-=0.2s").unwrap(),
            Position::LabelOffset(
                "intro".to_string(),
                SignedDuration::negative(Duration::from_millis(200))
            )
        );
    }
}
