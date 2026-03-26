//! Type system for block ports and compatibility checking
//!
//! Defines data types, port specifications, and compatibility rules.

use druid::Data;
use std::collections::HashSet;

/// Data types supported by block ports
#[derive(Data, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataType {
    /// Any type (generic)
    Any,
    /// Integer number
    Integer,
    /// Floating point number
    Float,
    /// String/text
    String,
    /// Boolean value
    Boolean,
    /// Array/list of values
    Array(Box<DataType>),
    /// Object/map/dictionary
    Object,
    /// Function/callable
    Function,
    /// Control flow (execution connection)
    ControlFlow,
    /// Custom named type
    Custom(String),
    /// Void/no return
    Void,
}

impl DataType {
    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &DataType) -> bool {
        match (self, other) {
            // Any type is compatible with everything
            (DataType::Any, _) | (_, DataType::Any) => true,
            
            // Same types are always compatible
            (a, b) if a == b => true,
            
            // Integer is compatible with Float (implicit conversion)
            (DataType::Integer, DataType::Float) |
            (DataType::Float, DataType::Integer) => true,
            
            // Array type compatibility
            (DataType::Array(a), DataType::Array(b)) => a.is_compatible_with(b),
            
            // Control flow only connects to control flow
            (DataType::ControlFlow, DataType::ControlFlow) => true,
            
            // Everything else is incompatible
            _ => false,
        }
    }
    
    /// Get display name for the type
    pub fn display_name(&self) -> String {
        match self {
            DataType::Any => "any".to_string(),
            DataType::Integer => "int".to_string(),
            DataType::Float => "float".to_string(),
            DataType::String => "string".to_string(),
            DataType::Boolean => "bool".to_string(),
            DataType::Array(inner) => format!("{}[]", inner.display_name()),
            DataType::Object => "object".to_string(),
            DataType::Function => "function".to_string(),
            DataType::ControlFlow => "exec".to_string(),
            DataType::Custom(name) => name.clone(),
            DataType::Void => "void".to_string(),
        }
    }
    
    /// Get color for visual representation
    pub fn color_key(&self) -> &'static str {
        match self {
            DataType::Any => "any",
            DataType::Integer => "int",
            DataType::Float => "float",
            DataType::String => "string",
            DataType::Boolean => "bool",
            DataType::Array(_) => "array",
            DataType::Object => "object",
            DataType::Function => "function",
            DataType::ControlFlow => "control",
            DataType::Custom(_) => "custom",
            DataType::Void => "void",
        }
    }
}

/// Port direction
#[derive(Data, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    /// Input port (receives data)
    Input,
    /// Output port (provides data)
    Output,
    /// Execution input (control flow)
    ExecInput,
    /// Execution output (control flow)
    ExecOutput,
}

impl PortDirection {
    pub fn is_input(&self) -> bool {
        matches!(self, PortDirection::Input | PortDirection::ExecInput)
    }
    
    pub fn is_output(&self) -> bool {
        matches!(self, PortDirection::Output | PortDirection::ExecOutput)
    }
    
    pub fn is_control(&self) -> bool {
        matches!(self, PortDirection::ExecInput | PortDirection::ExecOutput)
    }
    
    pub fn opposite(&self) -> PortDirection {
        match self {
            PortDirection::Input => PortDirection::Output,
            PortDirection::Output => PortDirection::Input,
            PortDirection::ExecInput => PortDirection::ExecOutput,
            PortDirection::ExecOutput => PortDirection::ExecInput,
        }
    }
}

/// Port specification
#[derive(Data, Clone, Debug)]
pub struct PortSpec {
    /// Unique port identifier within the block
    pub id: String,
    /// Display name
    pub name: String,
    /// Port direction
    pub direction: PortDirection,
    /// Data type
    pub data_type: DataType,
    /// Whether the port is required
    pub required: bool,
    /// Default value (for optional inputs)
    pub default_value: Option<String>,
    /// Description/tooltip
    pub description: String,
}

impl PortSpec {
    pub fn new(id: impl Into<String>, name: impl Into<String>, direction: PortDirection, data_type: DataType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            direction,
            data_type,
            required: true,
            default_value: None,
            description: String::new(),
        }
    }
    
    pub fn input(id: impl Into<String>, name: impl Into<String>, data_type: DataType) -> Self {
        Self::new(id, name, PortDirection::Input, data_type)
    }
    
    pub fn output(id: impl Into<String>, name: impl Into<String>, data_type: DataType) -> Self {
        Self::new(id, name, PortDirection::Output, data_type)
    }
    
    pub fn exec_input(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, PortDirection::ExecInput, DataType::ControlFlow)
    }
    
    pub fn exec_output(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, PortDirection::ExecOutput, DataType::ControlFlow)
    }
    
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
    
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self.required = false;
        self
    }
    
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
    
    /// Check if this port can connect to another port
    pub fn can_connect_to(&self, other: &PortSpec) -> bool {
        // Must be opposite directions
        if self.direction.is_input() == other.direction.is_input() {
            return false;
        }
        
        // Control flow ports only connect to control flow ports
        if self.direction.is_control() != other.direction.is_control() {
            return false;
        }
        
        // Check type compatibility
        self.data_type.is_compatible_with(&other.data_type)
    }
}

/// Port identifier (block_id, port_id)
#[derive(Data, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PortId {
    pub block_id: String,
    pub port_id: String,
}

impl PortId {
    pub fn new(block_id: impl Into<String>, port_id: impl Into<String>) -> Self {
        Self {
            block_id: block_id.into(),
            port_id: port_id.into(),
        }
    }
}

/// Connection validation result
#[derive(Debug, Clone)]
pub enum ConnectionValidation {
    /// Connection is valid
    Valid,
    /// Ports are incompatible
    IncompatibleTypes { source_type: DataType, target_type: DataType },
    /// Same direction ports
    SameDirection,
    /// Control/data mismatch
    ControlDataMismatch,
    /// Would create a cycle
    WouldCreateCycle,
    /// Target port already connected (for single-connection inputs)
    TargetAlreadyConnected,
    /// Self-connection not allowed
    SelfConnection,
}

impl ConnectionValidation {
    pub fn is_valid(&self) -> bool {
        matches!(self, ConnectionValidation::Valid)
    }
    
    pub fn error_message(&self) -> Option<String> {
        match self {
            ConnectionValidation::Valid => None,
            ConnectionValidation::IncompatibleTypes { source_type, target_type } => {
                Some(format!(
                    "Type mismatch: {} is not compatible with {}",
                    source_type.display_name(),
                    target_type.display_name()
                ))
            }
            ConnectionValidation::SameDirection => {
                Some("Cannot connect ports with the same direction".to_string())
            }
            ConnectionValidation::ControlDataMismatch => {
                Some("Cannot connect control flow to data ports".to_string())
            }
            ConnectionValidation::WouldCreateCycle => {
                Some("Connection would create a cycle".to_string())
            }
            ConnectionValidation::TargetAlreadyConnected => {
                Some("Target port is already connected".to_string())
            }
            ConnectionValidation::SelfConnection => {
                Some("Cannot connect a block to itself".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_compatibility() {
        assert!(DataType::Integer.is_compatible_with(&DataType::Integer));
        assert!(DataType::Integer.is_compatible_with(&DataType::Float));
        assert!(DataType::Any.is_compatible_with(&DataType::String));
        assert!(!DataType::String.is_compatible_with(&DataType::Boolean));
    }
    
    #[test]
    fn test_port_compatibility() {
        let output = PortSpec::output("out", "Result", DataType::Integer);
        let input = PortSpec::input("in", "Value", DataType::Float);
        assert!(output.can_connect_to(&input));
        
        let input2 = PortSpec::input("in2", "Text", DataType::String);
        assert!(!output.can_connect_to(&input2));
    }
}
