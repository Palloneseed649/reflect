use reflect_core::types::{EvalError, EvalSignal, Severity};
use regex::Regex;

pub fn parse_pytest_output(output: &str) -> EvalSignal {
    let passed = !summary_line(output)
        .map(|s| s.contains("failed"))
        .unwrap_or(false);
    let summary = extract_summary(output);
    let errors = if passed {
        vec![]
    } else {
        extract_errors(output)
    };

    EvalSignal {
        evaluator: "pytest".into(),
        passed,
        summary,
        errors,
    }
}

fn summary_line(output: &str) -> Option<&str> {
    output
        .lines()
        .rev()
        .find(|l| l.contains("passed") || l.contains("failed"))
        .filter(|l| l.starts_with('='))
}

fn extract_summary(output: &str) -> String {
    let re = Regex::new(r"={3,}\s+(.+?)\s+={3,}\s*$").unwrap();
    output
        .lines()
        .rev()
        .find_map(|line| {
            re.captures(line)
                .map(|c| c.get(1).unwrap().as_str().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

fn extract_errors(output: &str) -> Vec<EvalError> {
    // Build a map of file:line from the detailed failure blocks
    // Pattern: tests/file.py:LINE: ErrorType
    let line_re = Regex::new(r"^(\S+\.py):(\d+): \w+").unwrap();
    let mut file_lines: Vec<(String, u32)> = Vec::new();
    for line in output.lines() {
        if let Some(cap) = line_re.captures(line.trim()) {
            file_lines.push((cap[1].to_string(), cap[2].parse().unwrap()));
        }
    }

    // Extract errors from "short test summary info" section
    let summary_re =
        Regex::new(r"^FAILED\s+(\S+\.py)::\S+\s+-\s+(.+)$").unwrap();

    let mut errors = Vec::new();
    let in_summary = output
        .lines()
        .skip_while(|l| !l.contains("short test summary info"));

    for line in in_summary {
        if let Some(cap) = summary_re.captures(line.trim()) {
            let file = cap[1].to_string();
            let message = cap[2].to_string();

            // Look up line number from the detailed failure block
            let line_num = file_lines
                .iter()
                .find(|(f, _)| f == &file)
                .map(|(_, l)| *l);

            errors.push(EvalError {
                file: Some(file),
                line: line_num,
                column: None,
                code: None,
                message,
                severity: Severity::Error,
            });
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../../tests/fixtures/pytest_output.txt");

    #[test]
    fn parses_failure_output() {
        let signal = parse_pytest_output(FIXTURE);
        assert!(!signal.passed);
        assert_eq!(signal.evaluator, "pytest");
        assert!(signal.summary.contains("2 failed"));
        assert!(signal.summary.contains("3 passed"));
        assert_eq!(signal.errors.len(), 2);

        assert_eq!(signal.errors[0].file, Some("tests/test_math.py".into()));
        assert_eq!(signal.errors[0].line, Some(15));
        assert!(signal.errors[0].message.contains("ZeroDivisionError"));
        assert_eq!(signal.errors[0].severity, Severity::Error);

        assert_eq!(signal.errors[1].file, Some("tests/test_utils.py".into()));
        assert_eq!(signal.errors[1].line, Some(23));
        assert!(signal.errors[1].message.contains("ValueError"));
        assert_eq!(signal.errors[1].severity, Severity::Error);
    }

    #[test]
    fn parses_passing_output() {
        let output = "============================= test session starts ==============================\nplatform linux -- Python 3.12.1, pytest-8.1.1, pluggy-1.4.0\ncollected 3 items\n\ntests/test_math.py ...                                                   [100%]\n\n============================== 3 passed in 0.12s ==============================\n";
        let signal = parse_pytest_output(output);
        assert!(signal.passed);
        assert_eq!(signal.evaluator, "pytest");
        assert!(signal.summary.contains("3 passed"));
        assert!(signal.errors.is_empty());
    }
}
