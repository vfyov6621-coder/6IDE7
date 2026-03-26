//! Connection graph for managing block relationships
//!
//! Efficient graph structure for tracking connections and performing queries.

use crate::types::{DataType, PortDirection, PortId, PortSpec, ConnectionValidation};
use druid::Data;
use std::collections::{HashMap, HashSet, VecDeque};
use im::Vector;

/// A connection between two ports
#[derive(Data, Clone, Debug, PartialEq, Eq)]
pub struct Connection {
    /// Unique connection identifier
    pub id: String,
    /// Source port (output)
    pub source: PortId,
    /// Target port (input)
    pub target: PortId,
    /// Connection type (for visualization)
    pub connection_type: ConnectionType,
}

impl Connection {
    pub fn new(source: PortId, target: PortId, connection_type: ConnectionType) -> Self {
        Self {
            id: format!("{}:{}", source.block_id, target.block_id),
            source,
            target,
            connection_type,
        }
    }
}

/// Type of connection for visualization
#[derive(Data, Clone, Debug, PartialEq, Eq, Copy)]
pub enum ConnectionType {
    /// Data flow connection
    Data,
    /// Control flow connection
    Control,
}

/// Graph of block connections
#[derive(Data, Clone, Debug)]
pub struct ConnectionGraph {
    /// All connections
    pub connections: im::Vector<Connection>,
    
    /// Index: block_id -> outgoing connections
    outgoing: im::HashMap<String, im::Vector<usize>>,
    
    /// Index: block_id -> incoming connections  
    incoming: im::HashMap<String, im::Vector<usize>>,
    
    /// Index: target port -> connection index (for fast lookup)
    by_target: im::HashMap<String, usize>,
    
    /// Index: source port -> connection indices
    by_source: im::HashMap<String, im::Vector<usize>>,
}

impl Default for ConnectionGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionGraph {
    pub fn new() -> Self {
        Self {
            connections: im::Vector::new(),
            outgoing: im::HashMap::new(),
            incoming: im::HashMap::new(),
            by_target: im::HashMap::new(),
            by_source: im::HashMap::new(),
        }
    }
    
    /// Port key for indexing
    fn port_key(port: &PortId) -> String {
        format!("{}:{}", port.block_id, port.port_id)
    }
    
    /// Add a connection
    pub fn add_connection(&mut self, connection: Connection) {
        let idx = self.connections.len();
        let source_key = Self::port_key(&connection.source);
        let target_key = Self::port_key(&connection.target);
        
        // Add to main storage
        self.connections.push_back(connection.clone());
        
        // Update outgoing index
        self.outgoing
            .entry(connection.source.block_id.clone())
            .or_insert_with(im::Vector::new)
            .push_back(idx);
        
        // Update incoming index
        self.incoming
            .entry(connection.target.block_id.clone())
            .or_insert_with(im::Vector::new)
            .push_back(idx);
        
        // Update source port index
        self.by_source
            .entry(source_key)
            .or_insert_with(im::Vector::new)
            .push_back(idx);
        
        // Update target port index
        self.by_target.insert(target_key, idx);
    }
    
    /// Remove a connection by index
    pub fn remove_connection(&mut self, idx: usize) -> Option<Connection> {
        if idx >= self.connections.len() {
            return None;
        }
        
        let connection = self.connections.remove(idx);
        
        // Update indices (note: this is simplified, real impl would reindex)
        self.outgoing = im::HashMap::new();
        self.incoming = im::HashMap::new();
        self.by_source = im::HashMap::new();
        self.by_target = im::HashMap::new();
        
        // Rebuild indices
        for (new_idx, conn) in self.connections.iter().enumerate() {
            let source_key = Self::port_key(&conn.source);
            let target_key = Self::port_key(&conn.target);
            
            self.outgoing
                .entry(conn.source.block_id.clone())
                .or_insert_with(im::Vector::new)
                .push_back(new_idx);
            
            self.incoming
                .entry(conn.target.block_id.clone())
                .or_insert_with(im::Vector::new)
                .push_back(new_idx);
            
            self.by_source
                .entry(source_key)
                .or_insert_with(im::Vector::new)
                .push_back(new_idx);
            
            self.by_target.insert(target_key, new_idx);
        }
        
        Some(connection)
    }
    
    /// Remove all connections involving a block
    pub fn remove_block_connections(&mut self, block_id: &str) -> im::Vector<Connection> {
        let removed: im::Vector<Connection> = self.connections
            .iter()
            .filter(|c| c.source.block_id == block_id || c.target.block_id == block_id)
            .cloned()
            .collect();
        
        self.connections = self.connections
            .iter()
            .filter(|c| c.source.block_id != block_id && c.target.block_id != block_id)
            .cloned()
            .collect();
        
        // Rebuild indices
        self.rebuild_indices();
        
        removed
    }
    
    /// Rebuild all indices from connections
    fn rebuild_indices(&mut self) {
        self.outgoing = im::HashMap::new();
        self.incoming = im::HashMap::new();
        self.by_source = im::HashMap::new();
        self.by_target = im::HashMap::new();
        
        for (idx, conn) in self.connections.iter().enumerate() {
            let source_key = Self::port_key(&conn.source);
            let target_key = Self::port_key(&conn.target);
            
            self.outgoing
                .entry(conn.source.block_id.clone())
                .or_insert_with(im::Vector::new)
                .push_back(idx);
            
            self.incoming
                .entry(conn.target.block_id.clone())
                .or_insert_with(im::Vector::new)
                .push_back(idx);
            
            self.by_source
                .entry(source_key)
                .or_insert_with(im::Vector::new)
                .push_back(idx);
            
            self.by_target.insert(target_key, idx);
        }
    }
    
    /// Get all connections for a block
    pub fn get_block_connections(&self, block_id: &str) -> Vec<&Connection> {
        let outgoing = self.outgoing.get(block_id).cloned().unwrap_or_default();
        let incoming = self.incoming.get(block_id).cloned().unwrap_or_default();
        
        let mut result: Vec<&Connection> = Vec::new();
        for idx in outgoing.iter().chain(incoming.iter()) {
            if let Some(conn) = self.connections.get(*idx) {
                result.push(conn);
            }
        }
        result
    }
    
    /// Get connection to a target port
    pub fn get_connection_to(&self, port: &PortId) -> Option<&Connection> {
        let key = Self::port_key(port);
        self.by_target
            .get(&key)
            .and_then(|idx| self.connections.get(*idx))
    }
    
    /// Get connections from a source port
    pub fn get_connections_from(&self, port: &PortId) -> Vec<&Connection> {
        let key = Self::port_key(port);
        self.by_source
            .get(&key)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|idx| self.connections.get(*idx))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Check if connecting would create a cycle
    pub fn would_create_cycle(&self, source: &PortId, target: &PortId) -> bool {
        // BFS from target block to see if we can reach source block
        if source.block_id == target.block_id {
            return true;
        }
        
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        
        // Start from the source block and follow outgoing connections
        queue.push_back(source.block_id.clone());
        
        while let Some(current) = queue.pop_front() {
            if current == target.block_id {
                return true;
            }
            
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            
            // Follow outgoing connections
            if let Some(indices) = self.outgoing.get(&current) {
                for idx in indices {
                    if let Some(conn) = self.connections.get(*idx) {
                        if !visited.contains(&conn.target.block_id) {
                            queue.push_back(conn.target.block_id.clone());
                        }
                    }
                }
            }
        }
        
        false
    }
    
    /// Validate a potential connection
    pub fn validate_connection(
        &self,
        source: &PortId,
        source_spec: &PortSpec,
        target: &PortId,
        target_spec: &PortSpec,
    ) -> ConnectionValidation {
        // Check self-connection
        if source.block_id == target.block_id {
            return ConnectionValidation::SelfConnection;
        }
        
        // Check direction
        if source_spec.direction.is_input() || target_spec.direction.is_output() {
            return ConnectionValidation::SameDirection;
        }
        
        // Check control/data mismatch
        if source_spec.direction.is_control() != target_spec.direction.is_control() {
            return ConnectionValidation::ControlDataMismatch;
        }
        
        // Check type compatibility
        if !source_spec.data_type.is_compatible_with(&target_spec.data_type) {
            return ConnectionValidation::IncompatibleTypes {
                source_type: source_spec.data_type.clone(),
                target_type: target_spec.data_type.clone(),
            };
        }
        
        // Check if target already connected (for single-connection inputs)
        if self.get_connection_to(target).is_some() {
            return ConnectionValidation::TargetAlreadyConnected;
        }
        
        // Check for cycles (only for data flow, control flow can have loops)
        if !source_spec.direction.is_control() && self.would_create_cycle(source, target) {
            return ConnectionValidation::WouldCreateCycle;
        }
        
        ConnectionValidation::Valid
    }
    
    /// Get topological order of blocks for code generation
    pub fn topological_sort(&self, block_ids: &[String]) -> Vec<String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        
        // Initialize
        for block_id in block_ids {
            in_degree.insert(block_id.clone(), 0);
            adjacency.insert(block_id.clone(), Vec::new());
        }
        
        // Build adjacency list
        for conn in &self.connections {
            if let Some(degree) = in_degree.get_mut(&conn.target.block_id) {
                *degree += 1;
            }
            if let Some(neighbors) = adjacency.get_mut(&conn.source.block_id) {
                neighbors.push(conn.target.block_id.clone());
            }
        }
        
        // Kahn's algorithm
        let mut queue: VecDeque<String> = VecDeque::new();
        for (block_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(block_id.clone());
            }
        }
        
        let mut result = Vec::new();
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
    
    /// Get connection count
    pub fn len(&self) -> usize {
        self.connections.len()
    }
    
    /// Clear all connections
    pub fn clear(&mut self) {
        self.connections.clear();
        self.outgoing.clear();
        self.incoming.clear();
        self.by_source.clear();
        self.by_target.clear();
    }
}
