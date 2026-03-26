//! Block data models for 6IDE7
//!
//! Defines the structure, behavior, and library of code blocks.

use crate::types::{DataType, PortDirection, PortSpec, PortId};
use druid::{Data, Point, Size};

/// A code block in the visual editor
#[derive(Data, Clone, Debug)]
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
    /// Port specifications
    pub ports: im::Vector<PortSpec>,
    /// Code template with placeholders
    pub code_template: String,
    /// Documentation
    pub doc: String,
    /// Whether the block is currently being dragged
    pub is_dragging: bool,
    /// Whether the block is selected
    pub is_selected: bool,
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
            ports: im::Vector::new(),
            code_template: String::new(),
            doc: String::new(),
            is_dragging: false,
            is_selected: false,
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
    
    /// Add a port
    pub fn with_port(mut self, port: PortSpec) -> Self {
        self.ports.push_back(port);
        self.update_size();
        self
    }
    
    /// Add an input port
    pub fn with_input(mut self, id: impl Into<String>, name: impl Into<String>, data_type: DataType) -> Self {
        self.ports.push_back(PortSpec::input(id, name, data_type));
        self.update_size();
        self
    }
    
    /// Add an output port
    pub fn with_output(mut self, id: impl Into<String>, name: impl Into<String>, data_type: DataType) -> Self {
        self.ports.push_back(PortSpec::output(id, name, data_type));
        self.update_size();
        self
    }
    
    /// Add execution input
    pub fn with_exec_input(mut self, id: impl Into<String>, name: impl Into<String>) -> Self {
        self.ports.push_back(PortSpec::exec_input(id, name));
        self.update_size();
        self
    }
    
    /// Add execution output
    pub fn with_exec_output(mut self, id: impl Into<String>, name: impl Into<String>) -> Self {
        self.ports.push_back(PortSpec::exec_output(id, name));
        self.update_size();
        self
    }
    
    /// Set code template
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.code_template = template.into();
        self
    }
    
    /// Set documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = doc.into();
        self
    }
    
    /// Update block size based on ports
    fn update_size(&mut self) {
        let input_count = self.ports.iter().filter(|p| p.direction.is_input()).count();
        let output_count = self.ports.iter().filter(|p| p.direction.is_output()).count();
        let max_ports = input_count.max(output_count);
        
        // Base height + port spacing
        let header_height = 32.0;
        let port_spacing = 24.0;
        let padding = 16.0;
        
        let min_height = header_height + padding * 2.0 + 20.0;
        let calculated_height = header_height + (max_ports as f64) * port_spacing + padding;
        
        self.size.height = min_height.max(calculated_height);
    }
    
    /// Get input ports
    pub fn inputs(&self) -> impl Iterator<Item = &PortSpec> {
        self.ports.iter().filter(|p| p.direction.is_input())
    }
    
    /// Get output ports
    pub fn outputs(&self) -> impl Iterator<Item = &PortSpec> {
        self.ports.iter().filter(|p| p.direction.is_output())
    }
    
    /// Get port by ID
    pub fn get_port(&self, port_id: &str) -> Option<&PortSpec> {
        self.ports.iter().find(|p| p.id == port_id)
    }
    
    /// Get port position on the block (relative coordinates)
    pub fn get_port_position(&self, port: &PortSpec) -> Point {
        let header_height = 32.0;
        let port_spacing = 24.0;
        let padding = 12.0;
        
        let ports: Vec<_> = if port.direction.is_input() {
            self.inputs().collect()
        } else {
            self.outputs().collect()
        };
        
        let index = ports.iter().position(|p| p.id == port.id).unwrap_or(0);
        
        let y = header_height + padding + (index as f64) * port_spacing + 4.0;
        let x = if port.direction.is_input() {
            0.0
        } else {
            self.size.width
        };
        
        Point::new(x, y)
    }
    
    /// Get port position in canvas coordinates
    pub fn get_port_canvas_position(&self, port: &PortSpec) -> Point {
        let relative = self.get_port_position(port);
        Point::new(
            self.position.x + relative.x,
            self.position.y + relative.y,
        )
    }
    
    /// Check if a canvas point is inside this block
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.height
    }
    
    /// Get port at a canvas point
    pub fn get_port_at(&self, canvas_point: Point, port_radius: f64) -> Option<&PortSpec> {
        for port in &self.ports {
            let port_pos = self.get_port_canvas_position(port);
            let dx = canvas_point.x - port_pos.x;
            let dy = canvas_point.y - port_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance <= port_radius {
                return Some(port);
            }
        }
        None
    }
    
    /// Clone with a new ID
    pub fn clone_with_id(&self, new_id: impl Into<String>) -> Self {
        Self {
            id: new_id.into(),
            ..self.clone()
        }
    }
}

/// Block definition for the library (template)
#[derive(Clone, Debug)]
pub struct BlockDefinition {
    /// Block type identifier
    pub block_type: String,
    /// Display name
    pub name: String,
    /// Category
    pub category: String,
    /// Icon (emoji or symbol)
    pub icon: String,
    /// Ports
    pub ports: Vec<PortSpec>,
    /// Code template
    pub code_template: String,
    /// Documentation
    pub doc: String,
}

impl BlockDefinition {
    /// Create a block instance from this definition
    pub fn create_instance(&self, id: impl Into<String>, position: Point) -> BlockData {
        let mut block = BlockData::new(id, &self.name, &self.category);
        block.position = position;
        block.ports = self.ports.iter().cloned().collect();
        block.code_template = self.code_template.clone();
        block.doc = self.doc.clone();
        block.update_size();
        block
    }
}

/// Block library with predefined blocks
pub struct BlockLibrary;

impl BlockLibrary {
    /// Get all block definitions
    pub fn get_definitions() -> Vec<BlockDefinition> {
        let mut definitions = Vec::new();
        
        // === I/O BLOCKS ===
        
        definitions.push(BlockDefinition {
            block_type: "print".to_string(),
            name: "Print".to_string(),
            category: "io".to_string(),
            icon: "🖨".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("value", "Value", DataType::Any).description("Value to print"),
                PortSpec::exec_output("exec_out", "Next"),
            ],
            code_template: "print({{value}})".to_string(),
            doc: "Print a value to the console".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "input".to_string(),
            name: "Input".to_string(),
            category: "io".to_string(),
            icon: "⌨".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("prompt", "Prompt", DataType::String).default("\"\""),
                PortSpec::exec_output("exec_out", "Next"),
                PortSpec::output("result", "Result", DataType::String),
            ],
            code_template: "{{result}} = input({{prompt}})".to_string(),
            doc: "Read input from the user".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "read_file".to_string(),
            name: "Read File".to_string(),
            category: "io".to_string(),
            icon: "📄".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("path", "Path", DataType::String),
                PortSpec::exec_output("exec_out", "Next"),
                PortSpec::output("content", "Content", DataType::String),
            ],
            code_template: "{{content}} = open({{path}}).read()".to_string(),
            doc: "Read content from a file".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "write_file".to_string(),
            name: "Write File".to_string(),
            category: "io".to_string(),
            icon: "💾".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("path", "Path", DataType::String),
                PortSpec::input("content", "Content", DataType::String),
                PortSpec::exec_output("exec_out", "Next"),
            ],
            code_template: "open({{path}}, 'w').write({{content}})".to_string(),
            doc: "Write content to a file".to_string(),
        });
        
        // === DATA BLOCKS ===
        
        definitions.push(BlockDefinition {
            block_type: "variable".to_string(),
            name: "Variable".to_string(),
            category: "data".to_string(),
            icon: "📦".to_string(),
            ports: vec![
                PortSpec::input("value", "Value", DataType::Any),
                PortSpec::output("result", "Result", DataType::Any),
            ],
            code_template: "{{result}} = {{value}}".to_string(),
            doc: "Store a value in a variable".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "constant".to_string(),
            name: "Constant".to_string(),
            category: "data".to_string(),
            icon: "🔢".to_string(),
            ports: vec![
                PortSpec::output("value", "Value", DataType::Any),
            ],
            code_template: "{{value}}".to_string(),
            doc: "A constant value".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "array".to_string(),
            name: "Array".to_string(),
            category: "data".to_string(),
            icon: "📋".to_string(),
            ports: vec![
                PortSpec::input("items", "Items", DataType::Any).description("Comma-separated items"),
                PortSpec::output("array", "Array", DataType::Array(Box::new(DataType::Any))),
            ],
            code_template: "[{{items}}]".to_string(),
            doc: "Create an array".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "get_item".to_string(),
            name: "Get Item".to_string(),
            category: "data".to_string(),
            icon: "📖".to_string(),
            ports: vec![
                PortSpec::input("array", "Array", DataType::Array(Box::new(DataType::Any))),
                PortSpec::input("index", "Index", DataType::Integer),
                PortSpec::output("item", "Item", DataType::Any),
            ],
            code_template: "{{item}} = {{array}}[{{index}}]".to_string(),
            doc: "Get an item from an array by index".to_string(),
        });
        
        // === CONTROL BLOCKS ===
        
        definitions.push(BlockDefinition {
            block_type: "if".to_string(),
            name: "If".to_string(),
            category: "control".to_string(),
            icon: "❓".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("condition", "Condition", DataType::Boolean),
                PortSpec::exec_output("then", "Then"),
                PortSpec::exec_output("else", "Else"),
            ],
            code_template: "if {{condition}}:\n    {{then}}\nelse:\n    {{else}}".to_string(),
            doc: "Conditional branching".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "for_loop".to_string(),
            name: "For Loop".to_string(),
            category: "control".to_string(),
            icon: "🔁".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("start", "Start", DataType::Integer).default("0"),
                PortSpec::input("end", "End", DataType::Integer),
                PortSpec::input("step", "Step", DataType::Integer).default("1"),
                PortSpec::exec_output("loop", "Loop"),
                PortSpec::exec_output("done", "Done"),
                PortSpec::output("index", "Index", DataType::Integer),
            ],
            code_template: "for {{index}} in range({{start}}, {{end}}, {{step}}):\n    {{loop}}".to_string(),
            doc: "Loop a number of times".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "while_loop".to_string(),
            name: "While Loop".to_string(),
            category: "control".to_string(),
            icon: "🔄".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("condition", "Condition", DataType::Boolean),
                PortSpec::exec_output("loop", "Loop"),
                PortSpec::exec_output("done", "Done"),
            ],
            code_template: "while {{condition}}:\n    {{loop}}".to_string(),
            doc: "Loop while condition is true".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "for_each".to_string(),
            name: "For Each".to_string(),
            category: "control".to_string(),
            icon: "🔂".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("array", "Array", DataType::Array(Box::new(DataType::Any))),
                PortSpec::exec_output("loop", "Loop"),
                PortSpec::exec_output("done", "Done"),
                PortSpec::output("item", "Item", DataType::Any),
            ],
            code_template: "for {{item}} in {{array}}:\n    {{loop}}".to_string(),
            doc: "Iterate over array items".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "break".to_string(),
            name: "Break".to_string(),
            category: "control".to_string(),
            icon: "⏹".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
            ],
            code_template: "break".to_string(),
            doc: "Exit a loop".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "continue".to_string(),
            name: "Continue".to_string(),
            category: "control".to_string(),
            icon: "⏭".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
            ],
            code_template: "continue".to_string(),
            doc: "Skip to next iteration".to_string(),
        });
        
        // === FUNCTION BLOCKS ===
        
        definitions.push(BlockDefinition {
            block_type: "function_def".to_string(),
            name: "Function".to_string(),
            category: "function".to_string(),
            icon: "⚡".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("params", "Params", DataType::String).default(""),
                PortSpec::exec_output("body", "Body"),
                PortSpec::output("return", "Return", DataType::Any),
            ],
            code_template: "def function({{params}}):\n    {{body}}\n    return {{return}}".to_string(),
            doc: "Define a function".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "function_call".to_string(),
            name: "Call".to_string(),
            category: "function".to_string(),
            icon: "📞".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("function", "Function", DataType::Function),
                PortSpec::input("args", "Args", DataType::String).default(""),
                PortSpec::exec_output("exec_out", "Next"),
                PortSpec::output("result", "Result", DataType::Any),
            ],
            code_template: "{{result}} = {{function}}({{args}})".to_string(),
            doc: "Call a function".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "return".to_string(),
            name: "Return".to_string(),
            category: "function".to_string(),
            icon: "↩".to_string(),
            ports: vec![
                PortSpec::exec_input("exec_in", "Exec"),
                PortSpec::input("value", "Value", DataType::Any),
            ],
            code_template: "return {{value}}".to_string(),
            doc: "Return a value from a function".to_string(),
        });
        
        // === MATH BLOCKS ===
        
        definitions.push(BlockDefinition {
            block_type: "add".to_string(),
            name: "Add".to_string(),
            category: "math".to_string(),
            icon: "➕".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Any),
                PortSpec::input("b", "B", DataType::Any),
                PortSpec::output("result", "Result", DataType::Any),
            ],
            code_template: "({{a}} + {{b}})".to_string(),
            doc: "Add two values".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "subtract".to_string(),
            name: "Subtract".to_string(),
            category: "math".to_string(),
            icon: "➖".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Any),
                PortSpec::input("b", "B", DataType::Any),
                PortSpec::output("result", "Result", DataType::Any),
            ],
            code_template: "({{a}} - {{b}})".to_string(),
            doc: "Subtract two values".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "multiply".to_string(),
            name: "Multiply".to_string(),
            category: "math".to_string(),
            icon: "✖".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Any),
                PortSpec::input("b", "B", DataType::Any),
                PortSpec::output("result", "Result", DataType::Any),
            ],
            code_template: "({{a}} * {{b}})".to_string(),
            doc: "Multiply two values".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "divide".to_string(),
            name: "Divide".to_string(),
            category: "math".to_string(),
            icon: "➗".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Float),
                PortSpec::input("b", "B", DataType::Float),
                PortSpec::output("result", "Result", DataType::Float),
            ],
            code_template: "({{a}} / {{b}})".to_string(),
            doc: "Divide two values".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "modulo".to_string(),
            name: "Modulo".to_string(),
            category: "math".to_string(),
            icon: "٪".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Integer),
                PortSpec::input("b", "B", DataType::Integer),
                PortSpec::output("result", "Result", DataType::Integer),
            ],
            code_template: "({{a}} % {{b}})".to_string(),
            doc: "Modulo operation".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "power".to_string(),
            name: "Power".to_string(),
            category: "math".to_string(),
            icon: "x²".to_string(),
            ports: vec![
                PortSpec::input("base", "Base", DataType::Float),
                PortSpec::input("exp", "Exp", DataType::Float),
                PortSpec::output("result", "Result", DataType::Float),
            ],
            code_template: "({{base}} ** {{exp}})".to_string(),
            doc: "Exponentiation".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "compare".to_string(),
            name: "Compare".to_string(),
            category: "math".to_string(),
            icon: "⚖".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Any),
                PortSpec::input("b", "B", DataType::Any),
                PortSpec::output("result", "Result", DataType::Boolean),
            ],
            code_template: "({{a}} == {{b}})".to_string(),
            doc: "Compare two values for equality".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "greater".to_string(),
            name: "Greater".to_string(),
            category: "math".to_string(),
            icon: ">".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Float),
                PortSpec::input("b", "B", DataType::Float),
                PortSpec::output("result", "Result", DataType::Boolean),
            ],
            code_template: "({{a}} > {{b}})".to_string(),
            doc: "Check if A is greater than B".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "less".to_string(),
            name: "Less".to_string(),
            category: "math".to_string(),
            icon: "<".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Float),
                PortSpec::input("b", "B", DataType::Float),
                PortSpec::output("result", "Result", DataType::Boolean),
            ],
            code_template: "({{a}} < {{b}})".to_string(),
            doc: "Check if A is less than B".to_string(),
        });
        
        // === STRING BLOCKS ===
        
        definitions.push(BlockDefinition {
            block_type: "concat".to_string(),
            name: "Concat".to_string(),
            category: "string".to_string(),
            icon: "🔗".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::String),
                PortSpec::input("b", "B", DataType::String),
                PortSpec::output("result", "Result", DataType::String),
            ],
            code_template: "({{a}} + {{b}})".to_string(),
            doc: "Concatenate two strings".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "format".to_string(),
            name: "Format".to_string(),
            category: "string".to_string(),
            icon: "📝".to_string(),
            ports: vec![
                PortSpec::input("template", "Template", DataType::String),
                PortSpec::input("values", "Values", DataType::String),
                PortSpec::output("result", "Result", DataType::String),
            ],
            code_template: "{{template}}.format({{values}})".to_string(),
            doc: "Format a string with values".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "split".to_string(),
            name: "Split".to_string(),
            category: "string".to_string(),
            icon: "✂".to_string(),
            ports: vec![
                PortSpec::input("string", "String", DataType::String),
                PortSpec::input("delimiter", "Delimiter", DataType::String).default("\" \""),
                PortSpec::output("result", "Result", DataType::Array(Box::new(DataType::String))),
            ],
            code_template: "{{string}}.split({{delimiter}})".to_string(),
            doc: "Split a string by delimiter".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "join".to_string(),
            name: "Join".to_string(),
            category: "string".to_string(),
            icon: "📎".to_string(),
            ports: vec![
                PortSpec::input("array", "Array", DataType::Array(Box::new(DataType::String))),
                PortSpec::input("delimiter", "Delimiter", DataType::String).default("\" \""),
                PortSpec::output("result", "Result", DataType::String),
            ],
            code_template: "{{delimiter}}.join({{array}})".to_string(),
            doc: "Join array elements into string".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "length".to_string(),
            name: "Length".to_string(),
            category: "string".to_string(),
            icon: "📏".to_string(),
            ports: vec![
                PortSpec::input("value", "Value", DataType::Any),
                PortSpec::output("length", "Length", DataType::Integer),
            ],
            code_template: "len({{value}})".to_string(),
            doc: "Get the length of a string or array".to_string(),
        });
        
        // === LOGIC BLOCKS ===
        
        definitions.push(BlockDefinition {
            block_type: "and".to_string(),
            name: "And".to_string(),
            category: "math".to_string(),
            icon: "∧".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Boolean),
                PortSpec::input("b", "B", DataType::Boolean),
                PortSpec::output("result", "Result", DataType::Boolean),
            ],
            code_template: "({{a}} and {{b}})".to_string(),
            doc: "Logical AND".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "or".to_string(),
            name: "Or".to_string(),
            category: "math".to_string(),
            icon: "∨".to_string(),
            ports: vec![
                PortSpec::input("a", "A", DataType::Boolean),
                PortSpec::input("b", "B", DataType::Boolean),
                PortSpec::output("result", "Result", DataType::Boolean),
            ],
            code_template: "({{a}} or {{b}})".to_string(),
            doc: "Logical OR".to_string(),
        });
        
        definitions.push(BlockDefinition {
            block_type: "not".to_string(),
            name: "Not".to_string(),
            category: "math".to_string(),
            icon: "¬".to_string(),
            ports: vec![
                PortSpec::input("value", "Value", DataType::Boolean),
                PortSpec::output("result", "Result", DataType::Boolean),
            ],
            code_template: "(not {{value}})".to_string(),
            doc: "Logical NOT".to_string(),
        });
        
        definitions
    }
    
    /// Get block definition by type
    pub fn get_definition(block_type: &str) -> Option<BlockDefinition> {
        Self::get_definitions()
            .into_iter()
            .find(|d| d.block_type == block_type)
    }
    
    /// Get definitions by category
    pub fn get_by_category(category: &str) -> Vec<BlockDefinition> {
        Self::get_definitions()
            .into_iter()
            .filter(|d| d.category == category)
            .collect()
    }
    
    /// Get all categories
    pub fn get_categories() -> Vec<(&'static str, &'static str)> {
        vec![
            ("io", "Input/Output"),
            ("data", "Data"),
            ("control", "Control Flow"),
            ("function", "Functions"),
            ("math", "Math & Logic"),
            ("string", "Strings"),
        ]
    }
}
