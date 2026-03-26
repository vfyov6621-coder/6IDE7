//! Code generation module for 6IDE7
//!
//! Transforms block graphs into executable code in multiple languages.

pub mod python;
pub mod javascript;
pub mod rust;

use crate::blocks::BlockData;
use crate::graph::ConnectionGraph;
use crate::types::{DataType, PortDirection};
use druid::Data;

/// Supported target languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum TargetLanguage {
    Python,
    JavaScript,
    Rust,
}

impl TargetLanguage {
    pub fn extension(&self) -> &'static str {
        match self {
            TargetLanguage::Python => "py",
            TargetLanguage::JavaScript => "js",
            TargetLanguage::Rust => "rs",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetLanguage::Python => "Python",
            TargetLanguage::JavaScript => "JavaScript",
            TargetLanguage::Rust => "Rust",
        }
    }
    
    pub fn all() -> Vec<TargetLanguage> {
        vec![TargetLanguage::Python, TargetLanguage::JavaScript, TargetLanguage::Rust]
    }
}

/// Generated code result
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// Target language
    pub language: TargetLanguage,
    /// Generated source code
    pub code: String,
    /// Entry point function name (if applicable)
    pub entry_point: Option<String>,
    /// List of required imports/dependencies
    pub imports: Vec<String>,
    /// Any warnings during generation
    pub warnings: Vec<String>,
}

impl GeneratedCode {
    pub fn is_empty(&self) -> bool {
        self.code.trim().is_empty()
    }
    
    pub fn line_count(&self) -> usize {
        self.code.lines().count()
    }
}

/// Code generation context
#[derive(Debug)]
pub struct CodeContext {
    /// Current indentation level
    pub indent_level: usize,
    /// Indentation string (spaces or tabs)
    pub indent_str: String,
    /// Local variable counter for unique names
    pub var_counter: usize,
    /// Generated variable names: port_id -> var_name
    pub var_names: std::collections::HashMap<String, String>,
    /// Collected imports
    pub imports: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Output lines
    pub lines: Vec<String>,
}

impl CodeContext {
    pub fn new(indent_size: usize) -> Self {
        Self {
            indent_level: 0,
            indent_str: " ".repeat(indent_size),
            var_counter: 0,
            var_names: std::collections::HashMap::new(),
            imports: Vec::new(),
            warnings: Vec::new(),
            lines: Vec::new(),
        }
    }
    
    /// Generate a unique variable name
    pub fn fresh_var(&mut self, prefix: &str) -> String {
        self.var_counter += 1;
        format!("{}_{}", prefix, self.var_counter)
    }
    
    /// Get or create a variable name for a block output
    pub fn get_output_var(&mut self, block_id: &str, port_id: &str) -> String {
        let key = format!("{}:{}", block_id, port_id);
        if let Some(name) = self.var_names.get(&key) {
            return name.clone();
        }
        
        let name = self.fresh_var("result");
        self.var_names.insert(key, name.clone());
        name
    }
    
    /// Set variable name for an output
    pub fn set_output_var(&mut self, block_id: &str, port_id: &str, name: String) {
        let key = format!("{}:{}", block_id, port_id);
        self.var_names.insert(key, name);
    }
    
    /// Add an import statement
    pub fn add_import(&mut self, import: String) {
        if !self.imports.contains(&import) {
            self.imports.push(import);
        }
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        if !self.warnings.contains(&warning) {
            self.warnings.push(warning);
        }
    }
    
    /// Add a line of code
    pub fn add_line(&mut self, line: String) {
        self.lines.push(format!("{}{}", self.indent(), line));
    }
    
    /// Add raw line without indentation
    pub fn add_raw_line(&mut self, line: String) {
        self.lines.push(line);
    }
    
    /// Add empty line
    pub fn add_empty_line(&mut self) {
        self.lines.push(String::new());
    }
    
    /// Get indentation string for current level
    pub fn indent(&self) -> String {
        self.indent_str.repeat(self.indent_level)
    }
    
    /// Increase indentation
    pub fn push_indent(&mut self) {
        self.indent_level += 1;
    }
    
    /// Decrease indentation
    pub fn pop_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
    
    /// Get final code
    pub fn finalize(&self) -> String {
        self.lines.join("\n")
    }
}

/// Code generator trait for language-specific implementations
pub trait CodeGenerator {
    /// Generate code from blocks and connections
    fn generate(
        &self,
        blocks: &[BlockData],
        connections: &ConnectionGraph,
    ) -> GeneratedCode;
    
    /// Get the target language
    fn language(&self) -> TargetLanguage;
    
    /// Generate a comment
    fn comment(&self, text: &str) -> String;
    
    /// Generate a string literal
    fn string_literal(&self, value: &str) -> String;
    
    /// Generate a number literal
    fn number_literal(&self, value: &str, is_float: bool) -> String;
    
    /// Generate a boolean literal
    fn bool_literal(&self, value: bool) -> String;
}

/// Generate code for the specified language
pub fn generate_code(
    language: TargetLanguage,
    blocks: &[BlockData],
    connections: &ConnectionGraph,
) -> GeneratedCode {
    match language {
        TargetLanguage::Python => python::PythonGenerator.generate(blocks, connections),
        TargetLanguage::JavaScript => javascript::JavaScriptGenerator.generate(blocks, connections),
        TargetLanguage::Rust => rust::RustGenerator.generate(blocks, connections),
    }
}

/// Find the entry point block (first control flow block)
pub fn find_entry_block(blocks: &[BlockData], connections: &ConnectionGraph) -> Option<&BlockData> {
    // Find blocks with exec outputs that have no exec inputs connected
    for block in blocks {
        let has_exec_output = block.ports.iter().any(|p| 
            matches!(p.direction, PortDirection::ExecOutput)
        );
        
        if has_exec_output {
            // Check if this block's exec input is not connected
            let exec_input_connected = block.ports.iter()
                .filter(|p| matches!(p.direction, PortDirection::ExecInput))
                .any(|p| {
                    connections.get_connection_to(&crate::types::PortId::new(&block.id, &p.id)).is_some()
                });
            
            if !exec_input_connected {
                return Some(block);
            }
        }
    }
    
    // Fallback: return first block with exec output
    blocks.iter().find(|b| {
        b.ports.iter().any(|p| matches!(p.direction, PortDirection::ExecOutput))
    })
}

/// Get blocks in execution order using topological sort
pub fn get_execution_order(
    blocks: &[BlockData],
    connections: &ConnectionGraph,
) -> Vec<String> {
    let block_ids: Vec<String> = blocks.iter().map(|b| b.id.clone()).collect();
    connections.topological_sort(&block_ids)
}

/// Check if a block is a pure data block (no control flow)
pub fn is_data_block(block: &BlockData) -> bool {
    !block.ports.iter().any(|p| p.direction.is_control())
}
