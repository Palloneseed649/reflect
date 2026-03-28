use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single reflection — the atomic unit of learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    pub id: Uuid,
    pub task_description: String,
    pub draft: String,
    pub error_signals: Vec<EvalSignal>,
    pub critique: String,
    pub lesson: String,
    pub outcome: Outcome,
    pub pattern_id: Option<String>,
    pub tags: Vec<String>,
    pub confidence: f32,
    pub validation_count: u32,
    pub contradiction_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_recalled: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredReflection {
    pub reflection: Reflection,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Success,
    Failure,
    Partial,
}

impl Outcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Partial => "partial",
        }
    }
}

impl std::str::FromStr for Outcome {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "success" => Ok(Self::Success),
            "failure" => Ok(Self::Failure),
            "partial" => Ok(Self::Partial),
            other => Err(format!("unknown outcome: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub id: String,
    pub category: String,
    pub description: String,
    pub occurrences: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub reflection_ids: Vec<Uuid>,
    pub trend: Trend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalSignal {
    pub evaluator: String,
    pub passed: bool,
    pub summary: String,
    pub errors: Vec<EvalError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalError {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub code: Option<String>,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionStats {
    pub total_reflections: u64,
    pub by_outcome: OutcomeCounts,
    pub top_patterns: Vec<ErrorPattern>,
    pub top_tags: Vec<TagCount>,
    pub avg_confidence: f64,
    pub reflections_this_week: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeCounts {
    pub success: u64,
    pub failure: u64,
    pub partial: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCount {
    pub tag: String,
    pub count: u64,
}

pub fn confidence_score(validations: u32, contradictions: u32) -> f32 {
    let v = validations as f32;
    let c = contradictions as f32;
    (0.5 + (v - c) / (v + c + 2.0)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflection_serializes_roundtrip() {
        let r = Reflection {
            id: Uuid::now_v7(),
            task_description: "parse date".into(),
            draft: "input.parse().unwrap()".into(),
            error_signals: vec![],
            critique: "used unwrap".into(),
            lesson: "use Result".into(),
            outcome: Outcome::Failure,
            pattern_id: Some("rust-unwrap-on-parse".into()),
            tags: vec!["rust".into()],
            confidence: 0.5,
            validation_count: 0,
            contradiction_count: 0,
            created_at: Utc::now(),
            last_recalled: None,
        };
        let json = serde_json::to_string(&r).unwrap();
        let r2: Reflection = serde_json::from_str(&json).unwrap();
        assert_eq!(r.id, r2.id);
        assert_eq!(r.lesson, r2.lesson);
        assert_eq!(r.outcome, r2.outcome);
    }

    #[test]
    fn eval_signal_serializes_roundtrip() {
        let s = EvalSignal {
            evaluator: "cargo_test".into(),
            passed: false,
            summary: "1 failed".into(),
            errors: vec![EvalError {
                file: Some("src/main.rs".into()),
                line: Some(42),
                column: None,
                code: None,
                message: "panicked at unwrap".into(),
                severity: Severity::Error,
            }],
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: EvalSignal = serde_json::from_str(&json).unwrap();
        assert_eq!(s.evaluator, s2.evaluator);
        assert!(!s2.passed);
        assert_eq!(s2.errors.len(), 1);
        assert_eq!(s2.errors[0].line, Some(42));
    }

    #[test]
    fn outcome_display() {
        assert_eq!(Outcome::Failure.as_str(), "failure");
        assert_eq!(Outcome::Success.as_str(), "success");
        assert_eq!(Outcome::Partial.as_str(), "partial");
    }

    #[test]
    fn confidence_score_laplace() {
        assert!((confidence_score(0, 0) - 0.5).abs() < 0.001);
        assert!((confidence_score(2, 0) - 1.0).abs() < 0.001);
        assert!((confidence_score(0, 2) - 0.0).abs() < 0.001);
        assert!((confidence_score(3, 1) - 0.833).abs() < 0.01);
    }
}
