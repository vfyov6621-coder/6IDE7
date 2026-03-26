//! Code execution module for 6IDE7
//!
//! Executes generated code and captures output.

use std::process::Command;
use std::time::{Duration, Instant};

use crate::codegen::{GeneratedCode, TargetLanguage};

/// Execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution time
    pub duration: Duration,
    /// Whether execution was successful
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
}

impl ExecutionResult {
    pub fn success(stdout: String, duration: Duration) -> Self {
        Self {
            exit_code: Some(0),
            stdout,
            stderr: String::new(),
            duration,
            success: true,
            error: None,
        }
    }
    
    pub fn failure(stderr: String, error: Option<String>) -> Self {
        Self {
            exit_code: Some(1),
            stdout: String::new(),
            stderr,
            duration: Duration::ZERO,
            success: false,
            error,
        }
    }
    
    pub fn timeout() -> Self {
        Self {
            exit_code: None,
            stdout: String::new(),
            stderr: "Execution timed out".to_string(),
            duration: Duration::ZERO,
            success: false,
            error: Some("Execution exceeded time limit".to_string()),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.stdout.is_empty() && self.stderr.is_empty()
    }
}

/// Code executor
pub struct CodeExecutor {
    /// Timeout for execution
    timeout: Duration,
}

impl Default for CodeExecutor {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }
}

impl CodeExecutor {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
    
    /// Execute generated code
    pub fn execute(&self, code: &GeneratedCode) -> ExecutionResult {
        match code.language {
            TargetLanguage::Python => self.execute_python(code),
            TargetLanguage::JavaScript => self.execute_javascript(code),
            TargetLanguage::Rust => self.execute_rust(code),
        }
    }
    
    /// Execute Python code
    fn execute_python(&self, code: &GeneratedCode) -> ExecutionResult {
        let start = Instant::now();
        
        // Create temporary file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("6ide7_temp.py");
        
        if let Err(e) = std::fs::write(&temp_file, &code.code) {
            return ExecutionResult::failure(
                String::new(),
                Some(format!("Failed to write temp file: {}", e)),
            );
        }
        
        // Execute Python
        let result = Command::new("python3")
            .arg(&temp_file)
            .output();
        
        let duration = start.elapsed();
        
        // Cleanup
        let _ = std::fs::remove_file(&temp_file);
        
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                ExecutionResult {
                    exit_code: output.status.code(),
                    stdout,
                    stderr,
                    duration,
                    success: output.status.success(),
                    error: if output.status.success() {
                        None
                    } else {
                        Some("Execution failed".to_string())
                    },
                }
            }
            Err(e) => {
                ExecutionResult::failure(
                    String::new(),
                    Some(format!("Failed to execute Python: {}. Make sure Python 3 is installed.", e)),
                )
            }
        }
    }
    
    /// Execute JavaScript code
    fn execute_javascript(&self, code: &GeneratedCode) -> ExecutionResult {
        let start = Instant::now();
        
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("6ide7_temp.js");
        
        if let Err(e) = std::fs::write(&temp_file, &code.code) {
            return ExecutionResult::failure(
                String::new(),
                Some(format!("Failed to write temp file: {}", e)),
            );
        }
        
        let result = Command::new("node")
            .arg(&temp_file)
            .output();
        
        let duration = start.elapsed();
        let _ = std::fs::remove_file(&temp_file);
        
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                ExecutionResult {
                    exit_code: output.status.code(),
                    stdout,
                    stderr,
                    duration,
                    success: output.status.success(),
                    error: if output.status.success() { None } else { Some("Execution failed".to_string()) },
                }
            }
            Err(e) => {
                ExecutionResult::failure(
                    String::new(),
                    Some(format!("Failed to execute Node.js: {}. Make sure Node.js is installed.", e)),
                )
            }
        }
    }
    
    /// Execute Rust code
    fn execute_rust(&self, code: &GeneratedCode) -> ExecutionResult {
        let start = Instant::now();
        
        let temp_dir = std::env::temp_dir().join("6ide7_rust_project");
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            return ExecutionResult::failure(
                String::new(),
                Some(format!("Failed to create temp directory: {}", e)),
            );
        }
        
        let cargo_toml = r#"[package]
name = "ide7_temp"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        
        if let Err(e) = std::fs::write(temp_dir.join("Cargo.toml"), cargo_toml) {
            return ExecutionResult::failure(String::new(), Some(format!("Failed to write Cargo.toml: {}", e)));
        }
        
        let src_dir = temp_dir.join("src");
        if let Err(e) = std::fs::create_dir_all(&src_dir) {
            return ExecutionResult::failure(String::new(), Some(format!("Failed to create src directory: {}", e)));
        }
        
        if let Err(e) = std::fs::write(src_dir.join("main.rs"), &code.code) {
            return ExecutionResult::failure(String::new(), Some(format!("Failed to write main.rs: {}", e)));
        }
        
        // Build
        let build_result = Command::new("cargo")
            .current_dir(&temp_dir)
            .args(&["build", "--release"])
            .output();
        
        match build_result {
            Ok(build_output) => {
                if !build_output.status.success() {
                    let stderr = String::from_utf8_lossy(&build_output.stderr).to_string();
                    return ExecutionResult::failure(stderr, Some("Rust compilation failed".to_string()));
                }
            }
            Err(e) => {
                return ExecutionResult::failure(
                    String::new(),
                    Some(format!("Failed to run cargo: {}. Make sure Rust is installed.", e)),
                );
            }
        }
        
        // Run
        let binary_path = temp_dir.join("target/release/ide7_temp");
        let run_result = Command::new(&binary_path).output();
        
        let duration = start.elapsed();
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        match run_result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                ExecutionResult {
                    exit_code: output.status.code(),
                    stdout,
                    stderr,
                    duration,
                    success: output.status.success(),
                    error: if output.status.success() { None } else { Some("Execution failed".to_string()) },
                }
            }
            Err(e) => {
                ExecutionResult::failure(String::new(), Some(format!("Failed to run binary: {}", e)))
            }
        }
    }
}

/// Check if a language runtime is available
pub fn is_runtime_available(language: TargetLanguage) -> bool {
    let cmd = match language {
        TargetLanguage::Python => "python3",
        TargetLanguage::JavaScript => "node",
        TargetLanguage::Rust => "cargo",
    };
    
    Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get runtime version
pub fn get_runtime_version(language: TargetLanguage) -> Option<String> {
    let (cmd, args): (&str, &[&str]) = match language {
        TargetLanguage::Python => ("python3", &["--version"]),
        TargetLanguage::JavaScript => ("node", &["--version"]),
        TargetLanguage::Rust => ("cargo", &["--version"]),
    };
    
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
