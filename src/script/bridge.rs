//! Python Bridge - Rust <-> Python communication via PyO3
//!
//! Provides the interface for executing Python scripts and
//! exposing Fission's internal state to Python code.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use thiserror::Error;

/// Python bridge errors
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Python error: {0}")]
    PythonError(String),

    #[error("Script not found: {0}")]
    ScriptNotFound(String),

    #[error("Failed to initialize Python: {0}")]
    InitError(String),
}

/// Context passed to Python hooks
#[pyclass]
#[derive(Debug, Clone)]
pub struct HookContext {
    #[pyo3(get, set)]
    pub rax: u64,
    #[pyo3(get, set)]
    pub rbx: u64,
    #[pyo3(get, set)]
    pub rcx: u64,
    #[pyo3(get, set)]
    pub rdx: u64,
    #[pyo3(get, set)]
    pub rsi: u64,
    #[pyo3(get, set)]
    pub rdi: u64,
    #[pyo3(get, set)]
    pub rbp: u64,
    #[pyo3(get, set)]
    pub rsp: u64,
    #[pyo3(get, set)]
    pub rip: u64,
    #[pyo3(get, set)]
    pub r8: u64,
    #[pyo3(get, set)]
    pub r9: u64,
    #[pyo3(get, set)]
    pub r10: u64,
    #[pyo3(get, set)]
    pub r11: u64,
    #[pyo3(get, set)]
    pub r12: u64,
    #[pyo3(get, set)]
    pub r13: u64,
    #[pyo3(get, set)]
    pub r14: u64,
    #[pyo3(get, set)]
    pub r15: u64,
    #[pyo3(get, set)]
    pub rflags: u64,
}

#[pymethods]
impl HookContext {
    #[new]
    fn new() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            rip: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rflags: 0,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "HookContext(rip={:#x}, rax={:#x}, rbx={:#x}, rcx={:#x}, rdx={:#x})",
            self.rip, self.rax, self.rbx, self.rcx, self.rdx
        )
    }
}

impl Default for HookContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Fission API exposed to Python
#[pyclass]
pub struct FissionAPI {
    // Internal state references will be added here
}

#[pymethods]
impl FissionAPI {
    #[new]
    fn new() -> Self {
        Self {}
    }

    /// Print to the Fission console
    fn log(&self, message: &str) {
        println!("[Python] {}", message);
    }

    /// Read memory from the target process
    fn read_memory(&self, address: u64, size: usize) -> PyResult<Vec<u8>> {
        // TODO: Connect to actual memory manager
        log::debug!(
            "Python requested memory read: {:#x} ({} bytes)",
            address,
            size
        );
        Ok(vec![0u8; size])
    }

    /// Write memory to the target process
    fn write_memory(&self, address: u64, data: Vec<u8>) -> PyResult<usize> {
        // TODO: Connect to actual memory manager
        log::debug!(
            "Python requested memory write: {:#x} ({} bytes)",
            address,
            data.len()
        );
        Ok(data.len())
    }

    /// Set a breakpoint at the given address
    fn set_breakpoint(&self, address: u64) -> PyResult<bool> {
        log::debug!("Python set breakpoint: {:#x}", address);
        Ok(true)
    }

    /// Get the current instruction pointer
    fn get_rip(&self) -> PyResult<u64> {
        // TODO: Connect to actual debugger state
        Ok(0)
    }
}

/// Main Python bridge interface
pub struct PythonBridge {
    /// Whether Python has been initialized
    initialized: bool,
}

impl PythonBridge {
    /// Create a new Python bridge
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize the Python interpreter
    pub fn initialize(&mut self) -> Result<(), ScriptError> {
        if self.initialized {
            return Ok(());
        }

        pyo3::prepare_freethreaded_python();
        self.initialized = true;

        log::info!("Python interpreter initialized");
        Ok(())
    }

    /// Execute a Python script string
    pub fn execute(&self, code: &str) -> Result<String, ScriptError> {
        if !self.initialized {
            return Err(ScriptError::InitError("Python not initialized".into()));
        }

        Python::with_gil(|py| {
            // Create Fission module
            let fission = PyModule::new(py, "fission")
                .map_err(|e| ScriptError::PythonError(e.to_string()))?;

            fission
                .add_class::<FissionAPI>()
                .map_err(|e| ScriptError::PythonError(e.to_string()))?;
            fission
                .add_class::<HookContext>()
                .map_err(|e| ScriptError::PythonError(e.to_string()))?;

            // Add fission module to sys.modules
            let sys = py
                .import("sys")
                .map_err(|e| ScriptError::PythonError(e.to_string()))?;
            let modules: &PyDict = sys
                .getattr("modules")
                .map_err(|e| ScriptError::PythonError(e.to_string()))?
                .downcast()
                .map_err(|e| ScriptError::PythonError(e.to_string()))?;
            modules
                .set_item("fission", fission)
                .map_err(|e| ScriptError::PythonError(e.to_string()))?;

            // Execute the code
            let result = py.eval(code, None, None);

            match result {
                Ok(val) => Ok(val.to_string()),
                Err(e) => Err(ScriptError::PythonError(e.to_string())),
            }
        })
    }

    /// Execute a Python file
    pub fn execute_file(&self, path: &str) -> Result<(), ScriptError> {
        let code = std::fs::read_to_string(path)
            .map_err(|_| ScriptError::ScriptNotFound(path.to_string()))?;

        self.run(&code)?;
        Ok(())
    }

    /// Run a Python code block (multi-line)
    pub fn run(&self, code: &str) -> Result<(), ScriptError> {
        if !self.initialized {
            return Err(ScriptError::InitError("Python not initialized".into()));
        }

        Python::with_gil(|py| {
            py.run(code, None, None)
                .map_err(|e| ScriptError::PythonError(e.to_string()))
        })
    }

    /// Call a Python function with a HookContext
    pub fn call_hook(&self, func_name: &str, ctx: &mut HookContext) -> Result<(), ScriptError> {
        if !self.initialized {
            return Err(ScriptError::InitError("Python not initialized".into()));
        }

        Python::with_gil(|py| {
            let globals = PyDict::new(py);
            globals
                .set_item("ctx", ctx.clone())
                .map_err(|e| ScriptError::PythonError(e.to_string()))?;

            let call_code = format!("{}(ctx)", func_name);
            py.run(&call_code, Some(globals), None)
                .map_err(|e| ScriptError::PythonError(e.to_string()))?;

            // Update context from Python modifications
            if let Ok(new_ctx) = globals.get_item("ctx") {
                if let Some(new_ctx) = new_ctx {
                    if let Ok(updated) = new_ctx.extract::<HookContext>() {
                        *ctx = updated;
                    }
                }
            }

            Ok(())
        })
    }
}

impl Default for PythonBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_context() {
        let mut ctx = HookContext::new();
        ctx.rax = 0xDEADBEEF;
        assert_eq!(ctx.rax, 0xDEADBEEF);
    }
}
