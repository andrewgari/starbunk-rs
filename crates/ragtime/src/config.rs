//! Runtime configuration for ragtime.

use crate::fuzzy::DEFAULT_GRADE_THRESHOLD;

/// Env var overriding the answer similarity threshold.
pub const THRESHOLD_ENV: &str = "RAGTIME_SIMILARITY_THRESHOLD";

/// Ragtime settings.
#[derive(Debug, Clone, PartialEq)]
pub struct RagtimeConfig {
    /// Minimum similarity (`0.0..=1.0`) for a fuzzy answer to be accepted.
    pub similarity_threshold: f64,
}

impl Default for RagtimeConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: DEFAULT_GRADE_THRESHOLD,
        }
    }
}

impl RagtimeConfig {
    /// Build from an optional raw threshold string; `None` uses the default.
    pub fn from_threshold_str(raw: Option<&str>) -> anyhow::Result<Self> {
        let Some(raw) = raw else {
            return Ok(Self::default());
        };
        let value: f64 = raw
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("{THRESHOLD_ENV} must be a number, got {raw:?}"))?;
        if !(0.0..=1.0).contains(&value) {
            anyhow::bail!("{THRESHOLD_ENV} must be within 0.0..=1.0, got {value}");
        }
        Ok(Self {
            similarity_threshold: value,
        })
    }

    /// Load from the environment.
    pub fn from_env() -> anyhow::Result<Self> {
        Self::from_threshold_str(std::env::var(THRESHOLD_ENV).ok().as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_0_85() {
        assert!((RagtimeConfig::default().similarity_threshold - 0.85).abs() < f64::EPSILON);
        let c = RagtimeConfig::from_threshold_str(None).unwrap();
        assert_eq!(c, RagtimeConfig::default());
    }

    #[test]
    fn override_parses() {
        let c = RagtimeConfig::from_threshold_str(Some(" 0.7 ")).unwrap();
        assert!((c.similarity_threshold - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_rejected() {
        assert!(RagtimeConfig::from_threshold_str(Some("abc")).is_err());
        assert!(RagtimeConfig::from_threshold_str(Some("1.5")).is_err());
        assert!(RagtimeConfig::from_threshold_str(Some("-0.1")).is_err());
        assert!(RagtimeConfig::from_threshold_str(Some("NaN")).is_err());
    }
}
