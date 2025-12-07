//! WebAssembly bindings for Sui

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::interpreter::Interpreter;

#[cfg(feature = "wasm")]
use crate::transpiler::{Sui2Py, Sui2Js};

/// WebAssembly bindings for the Sui interpreter
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmSui {
    interpreter: Interpreter,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmSui {
    /// Create a new Sui interpreter
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    /// Run Sui code and return output as JSON array
    #[wasm_bindgen]
    pub fn run(&mut self, code: &str) -> Result<String, JsValue> {
        let output = self
            .interpreter
            .run(code, &[])
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Return as JSON array
        let json = serde_json::to_string(&output)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(json)
    }

    /// Run Sui code with arguments
    #[wasm_bindgen]
    pub fn run_with_args(&mut self, code: &str, args: &str) -> Result<String, JsValue> {
        // Parse args as JSON array
        let args: Vec<String> = serde_json::from_str(args)
            .map_err(|e| JsValue::from_str(&format!("Invalid args JSON: {}", e)))?;

        let output = self
            .interpreter
            .run(code, &args)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let json = serde_json::to_string(&output)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(json)
    }

    /// Reset the interpreter state
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.interpreter.reset();
    }

    /// Transpile Sui code to Python
    #[wasm_bindgen]
    pub fn to_python(code: &str) -> Result<String, JsValue> {
        let mut transpiler = Sui2Py::new();
        transpiler
            .transpile_to_python(code)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Transpile Sui code to JavaScript
    #[wasm_bindgen]
    pub fn to_javascript(code: &str) -> Result<String, JsValue> {
        let mut transpiler = Sui2Js::new();
        transpiler
            .transpile_to_js(code)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get Sui language version
    #[wasm_bindgen]
    pub fn version() -> String {
        crate::VERSION.to_string()
    }
}

#[cfg(feature = "wasm")]
impl Default for WasmSui {
    fn default() -> Self {
        Self::new()
    }
}
