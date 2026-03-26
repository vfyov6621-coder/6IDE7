//! Block data models for 6IDE7
//!
//! Defines the structure and behavior of code blocks.

use druid::{Data, Point, Size};

/// A code block in the visual editor
#[derive(Data, Clone)]
pub struct BlockData {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Category (io, data, control, function, math, string)
    pub category: String,
    /// Position on canvas
    pub position: Point,
    /// Size of the block
    pub size: Size,
    /// Input port names
    pub inputs: im::Vector<String>,
    /// Output port names
    pub outputs: im::Vector<String>,
    /// Code template
    pub code_template: String,
    /// Whether the block is currently being dragged
    pub is_dragging: bool,
}

impl BlockData {
    /// Create a new block with default settings
    pub fn new(id: impl Into<String>, name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category: category.into(),
            position: Point::ZERO,
            size: Size::new(160.0, 100.0),
            inputs: im::Vector::new(),
            outputs: im::Vector::new(),
            code_template: String::new(),
            is_dragging: false,
        }
    }
    
    /// Set block position
    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.position = Point::new(x, y);
        self
    }
    
    /// Set block size
    pub fn with_size(mut self, width: f64, height: f64) -> Self {
        self.size = Size::new(width, height);
        self
    }
    
    /// Add input port
    pub fn with_input(mut self, name: impl Into<String>) -> Self {
        self.inputs.push_back(name.into());
        self
    }
    
    /// Add output port
    pub fn with_output(mut self, name: impl Into<String>) -> Self {
        self.outputs.push_back(name.into());
        self
    }
    
    /// Set code template
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.code_template = template.into();
        self
    }
    
    /// Check if a canvas point is inside this block
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.height
    }
}

/// Connection between two blocks
#[derive(Data, Clone)]
pub struct Connection {
    /// Source block ID
    pub from_block: String,
    /// Source output port index
    pub from_port: usize,
    /// Target block ID
    pub to_block: String,
    /// Target input port index
    pub to_port: usize,
}

impl Connection {
    pub fn new(
        from_block: impl Into<String>,
        from_port: usize,
        to_block: impl Into<String>,
        to_port: usize,
    ) -> Self {
        Self {
            from_block: from_block.into(),
            from_port,
            to_block: to_block.into(),
            to_port,
        }
    }
}

/// Block library with predefined blocks
pub struct BlockLibrary;

impl BlockLibrary {
    /// Get all built-in blocks
    pub fn get_builtin_blocks() -> im::Vector<BlockData> {
        let mut blocks = im::Vector::new();
        
        // I/O blocks
        blocks.push_back(
            BlockData::new("print", "Print", "io")
                .with_input("value")
                .with_template("print({{value}})")
        );
        
        blocks.push_back(
            BlockData::new("read", "Read Input", "io")
                .with_output("result")
                .with_template("input()")
        );
        
        // Data blocks
        blocks.push_back(
            BlockData::new("variable", "Variable", "data")
                .with_input("value")
                .with_output("result")
                .with_template("{{name}} = {{value}}")
        );
        
        blocks.push_back(
            BlockData::new("constant", "Constant", "data")
                .with_output("value")
                .with_template("{{value}}")
        );
        
        // Control blocks
        blocks.push_back(
            BlockData::new("if", "If", "control")
                .with_input("condition")
                .with_output("then")
                .with_output("else")
                .with_size(160.0, 140.0)
                .with_template("if {{condition}}:\n    {{then}}\nelse:\n    {{else}}")
        );
        
        blocks.push_back(
            BlockData::new("for", "For Loop", "control")
                .with_input("start")
                .with_input("end")
                .with_output("body")
                .with_size(160.0, 120.0)
                .with_template("for i in range({{start}}, {{end}}):\n    {{body}}")
        );
        
        blocks.push_back(
            BlockData::new("while", "While Loop", "control")
                .with_input("condition")
                .with_output("body")
                .with_size(160.0, 120.0)
                .with_template("while {{condition}}:\n    {{body}}")
        );
        
        // Math blocks
        blocks.push_back(
            BlockData::new("add", "Add", "math")
                .with_input("a")
                .with_input("b")
                .with_output("result")
                .with_template("{{a}} + {{b}}")
        );
        
        blocks.push_back(
            BlockData::new("subtract", "Subtract", "math")
                .with_input("a")
                .with_input("b")
                .with_output("result")
                .with_template("{{a}} - {{b}}")
        );
        
        blocks.push_back(
            BlockData::new("multiply", "Multiply", "math")
                .with_input("a")
                .with_input("b")
                .with_output("result")
                .with_template("{{a}} * {{b}}")
        );
        
        blocks.push_back(
            BlockData::new("divide", "Divide", "math")
                .with_input("a")
                .with_input("b")
                .with_output("result")
                .with_template("{{a}} / {{b}}")
        );
        
        // String blocks
        blocks.push_back(
            BlockData::new("concat", "Concatenate", "string")
                .with_input("a")
                .with_input("b")
                .with_output("result")
                .with_template("{{a}} + {{b}}")
        );
        
        blocks.push_back(
            BlockData::new("format", "Format String", "string")
                .with_input("template")
                .with_input("values")
                .with_output("result")
                .with_template("{{template}}.format({{values}})")
        );
        
        // Function blocks
        blocks.push_back(
            BlockData::new("function", "Function", "function")
                .with_input("params")
                .with_output("body")
                .with_output("return")
                .with_size(160.0, 140.0)
                .with_template("def {{name}}({{params}}):\n    {{body}}\n    return {{return}}")
        );
        
        blocks.push_back(
            BlockData::new("call", "Call Function", "function")
                .with_input("args")
                .with_output("result")
                .with_template("{{name}}({{args}})")
        );
        
        blocks
    }
}
