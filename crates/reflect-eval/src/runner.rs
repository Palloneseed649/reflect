use crate::parsers;
use reflect_core::error::Result;
use reflect_core::types::EvalSignal;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub timeout: Duration,
    pub working_dir: String,
}

pub async fn run_evaluator(config: &RunnerConfig) -> Result<EvalSignal> {
    let result = tokio::time::timeout(config.timeout, async {
        Command::new("sh")
            .arg("-c")
            .arg(&config.command)
            .current_dir(&config.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
    })
    .await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{stdout}\n{stderr}");
            let signal = match config.name.as_str() {
                "cargo_test" => parsers::parse_cargo_test_output(&combined),
                "pytest" => parsers::parse_pytest_output(&combined),
                "eslint" => parsers::parse_eslint_output(&combined),
                "tsc" => parsers::parse_tsc_output(&combined),
                _ => EvalSignal {
                    evaluator: config.name.clone(),
                    passed: output.status.success(),
                    summary: if output.status.success() {
                        "passed".into()
                    } else {
                        format!("exit code {}", output.status.code().unwrap_or(-1))
                    },
                    errors: vec![],
                },
            };
            Ok(signal)
        }
        Ok(Err(e)) => Ok(EvalSignal {
            evaluator: config.name.clone(),
            passed: false,
            summary: format!("failed to execute: {e}"),
            errors: vec![],
        }),
        Err(_) => Ok(EvalSignal {
            evaluator: config.name.clone(),
            passed: false,
            summary: format!("timed out after {}s", config.timeout.as_secs()),
            errors: vec![],
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_echo_command() {
        let config = RunnerConfig {
            name: "custom".into(),
            command: "echo hello".into(),
            args: vec![],
            timeout: Duration::from_secs(5),
            working_dir: "/tmp".into(),
        };
        let signal = run_evaluator(&config).await.unwrap();
        assert!(signal.passed);
    }

    #[tokio::test]
    async fn run_failing_command() {
        let config = RunnerConfig {
            name: "custom".into(),
            command: "exit 1".into(),
            args: vec![],
            timeout: Duration::from_secs(5),
            working_dir: "/tmp".into(),
        };
        let signal = run_evaluator(&config).await.unwrap();
        assert!(!signal.passed);
    }

    #[tokio::test]
    async fn dispatch_pytest_parser() {
        let config = RunnerConfig {
            name: "pytest".into(),
            command: "echo 'FAILED tests/test_a.py::test_x - ValueError: bad\n============================== 1 failed, 0 passed in 0.1s =============================='".into(),
            args: vec![],
            timeout: Duration::from_secs(5),
            working_dir: "/tmp".into(),
        };
        let signal = run_evaluator(&config).await.unwrap();
        assert_eq!(signal.evaluator, "pytest");
        assert!(!signal.passed);
    }

    #[tokio::test]
    async fn dispatch_eslint_parser() {
        let config = RunnerConfig {
            name: "eslint".into(),
            command: "printf '  1:1  error  msg  no-undef\\n\\n✖ 1 problem (1 error, 0 warnings)\\n'".into(),
            args: vec![],
            timeout: Duration::from_secs(5),
            working_dir: "/tmp".into(),
        };
        let signal = run_evaluator(&config).await.unwrap();
        assert_eq!(signal.evaluator, "eslint");
        assert!(!signal.passed);
    }

    #[tokio::test]
    async fn dispatch_tsc_parser() {
        let config = RunnerConfig {
            name: "tsc".into(),
            command: "echo 'src/a.ts(1,1): error TS2304: Cannot find name x.'".into(),
            args: vec![],
            timeout: Duration::from_secs(5),
            working_dir: "/tmp".into(),
        };
        let signal = run_evaluator(&config).await.unwrap();
        assert_eq!(signal.evaluator, "tsc");
        assert!(!signal.passed);
    }
}
