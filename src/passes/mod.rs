pub mod pass_imports;

use super::app::App;
use crate::treesitter;
use std::fmt;
use std::process::{Child, Command, ExitStatus};
use std::rc::Rc;
use std::time::Duration;
use tree_sitter::Node as TSNode;
use wait_timeout::ChildExt;

/// Outcomes for test frameworks defined at POSIX 1003.3.
/// See: POSIX 1003.3: 1.4 A POSIX compliant test framework.
#[derive(Debug)]
pub enum TestOutcome {
    /// Test succeeds
    Pass,
    /// Test produced a failure
    Fail,
    /// Test produced intermediate results
    Unresolved,
}

impl fmt::Display for TestOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait Pass<'a> {
    /// Returns absolute path to the temporary file created for this pass.
    fn next_temp_file(&self) -> String;

    /// Returns temporary directory used by this pass.
    fn temp_dir(&self) -> String;

    /// Returns name of this pass.
    fn name(&self) -> String;

    /// Returns application configuration used by this pass.
    fn app(&self) -> &App;

    /// Returns the original source code of the program.
    fn original_source(&self) -> String;

    /// Returns tree-sitter parser.
    fn language(&self) -> Rc<dyn treesitter::Parser>;

    /// Returns the result of the execution of the check script. The source code for test will be
    /// generated from the given `source_code`, from which the `removed_nodes` are removed.
    fn test_nodes(
        &self,
        source_code: &str,
        removed_nodes: &[TSNode<'a>],
    ) -> Result<(TestOutcome, String), String> {
        let source = match self.language().remove_nodes(source_code, removed_nodes) {
            Ok(source) => source,
            Err(err) => return Err(err),
        };
        self.test_source(&source)
    }

    /// Returns the result of the execution of the check script for the source code.
    fn test_source(&self, source: &str) -> Result<(TestOutcome, String), String> {
        let temp_file = self.next_temp_file();
        let temp_file = temp_file.as_str();
        if std::fs::write(temp_file, source).is_err() {
            return Err("Cannot write to file".to_string());
        };
        let result = run_command(
            self.app().script.as_str(),
            self.app().timeout,
            vec![temp_file],
        );
        log::debug!("File: {} Result: {}", &temp_file, &result);
        Ok((result, source.to_string()))
    }
}

/// Wait wraps over `wait` function implementing timeout logic.
fn wait(child: &mut Child, timeout: Option<u32>) -> std::io::Result<ExitStatus> {
    match timeout {
        None => child.wait(),
        Some(timeout) => {
            let time_sec = Duration::from_secs(timeout as u64);
            match child.wait_timeout(time_sec).unwrap() {
                Some(s) => Ok(s),
                None => {
                    // Time out. Kill the child and return status code which will contain the
                    // error.
                    child.kill().unwrap();
                    Ok(child.wait().unwrap())
                }
            }
        }
    }
}

/// Executes the given shell command and returns TestOutcome::PASS if it returns 0 return code.
fn run_command(script: &str, timeout: Option<u32>, args: Vec<&str>) -> TestOutcome {
    if let Ok(mut child) = Command::new(script).args(&args).spawn() {
        match wait(&mut child, timeout) {
            Ok(s) => {
                if s.success() {
                    TestOutcome::Pass
                } else {
                    TestOutcome::Fail
                }
            }
            _ => TestOutcome::Unresolved,
        }
    } else {
        TestOutcome::Unresolved
    }
}
