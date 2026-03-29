use reflect_core::types::{EvalError, EvalSignal, Severity};
use regex::Regex;

pub fn parse_cargo_test_output(output: &str) -> EvalSignal {
    let passed = output.contains("test result: ok");
    let summary = extract_summary(output);
    let errors = if passed { vec![] } else { extract_errors(output) };
    EvalSignal {
        evaluator: "cargo_test".into(),
        passed,
        summary,
        errors,
    }
}

fn extract_summary(output: &str) -> String {
    let re = Regex::new(r"test result: \w+\.\s+(\d+ passed; \d+ failed)").unwrap();
    re.captures(output)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn extract_errors(output: &str) -> Vec<EvalError> {
    let mut errors = Vec::new();
    // Old format: panicked at 'MSG', FILE:LINE:COL
    let re1 = Regex::new(r"panicked at '([^']+)',\s+([^:\s]+):(\d+):(\d+)").unwrap();
    // New format: panicked at FILE:LINE:COL:\nMSG
    let re2 = Regex::new(r"panicked at ([^:\s]+):(\d+):(\d+):\s*\n\s*(.+)").unwrap();

    for cap in re1.captures_iter(output) {
        errors.push(EvalError {
            file: Some(cap[2].to_string()),
            line: cap[3].parse().ok(),
            column: cap[4].parse().ok(),
            code: None,
            message: cap[1].to_string(),
            severity: Severity::Error,
        });
    }
    for cap in re2.captures_iter(output) {
        errors.push(EvalError {
            file: Some(cap[1].to_string()),
            line: cap[2].parse().ok(),
            column: cap[3].parse().ok(),
            code: None,
            message: cap[4].to_string(),
            severity: Severity::Error,
        });
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../../tests/fixtures/cargo_test_output.txt");

    #[test]
    fn parses_failure_output() {
        let signal = parse_cargo_test_output(FIXTURE);
        assert!(!signal.passed);
        assert_eq!(signal.evaluator, "cargo_test");
        assert!(signal.summary.contains("2 passed"));
        assert!(signal.summary.contains("1 failed"));
        assert_eq!(signal.errors.len(), 1);
        assert_eq!(signal.errors[0].file, Some("src/lib.rs".into()));
        assert_eq!(signal.errors[0].line, Some(42));
        assert!(signal.errors[0].message.contains("unwrap()"));
    }

    #[test]
    fn parses_passing_output() {
        let output = "running 3 tests\ntest tests::a ... ok\ntest tests::b ... ok\ntest tests::c ... ok\n\ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        let signal = parse_cargo_test_output(output);
        assert!(signal.passed);
        assert!(signal.errors.is_empty());
    }
}
