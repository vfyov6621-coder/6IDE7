//! State management with Undo/Redo support
//!
//! History tracking for canvas operations.

use crate::canvas::CanvasState;
use druid::Data;
use std::collections::VecDeque;

/// Maximum history size
const MAX_HISTORY: usize = 50;

/// State snapshot for undo/redo
#[derive(Data, Clone)]
pub struct StateSnapshot {
    /// Canvas state
    pub canvas: CanvasState,
    /// Description of the action
    pub action: String,
}

/// History manager for undo/redo
#[derive(Data, Clone)]
pub struct HistoryManager {
    /// Undo stack
    undo_stack: im::Vector<StateSnapshot>,
    /// Redo stack  
    redo_stack: im::Vector<StateSnapshot>,
    /// Maximum history size
    max_size: usize,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new(MAX_HISTORY)
    }
}

impl HistoryManager {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: im::Vector::new(),
            redo_stack: im::Vector::new(),
            max_size,
        }
    }
    
    /// Push a new state onto the history
    pub fn push(&mut self, snapshot: StateSnapshot) {
        // Clear redo stack when new action is performed
        self.redo_stack.clear();
        
        // Limit history size
        if self.undo_stack.len() >= self.max_size {
            self.undo_stack.remove(0);
        }
        
        self.undo_stack.push_back(snapshot);
    }
    
    /// Undo the last action
    pub fn undo(&mut self, current: CanvasState) -> Option<CanvasState> {
        if let Some(snapshot) = self.undo_stack.pop_back() {
            // Save current state to redo stack
            self.redo_stack.push_back(StateSnapshot {
                canvas: current,
                action: snapshot.action.clone(),
            });
            Some(snapshot.canvas)
        } else {
            None
        }
    }
    
    /// Redo the last undone action
    pub fn redo(&mut self, current: CanvasState) -> Option<CanvasState> {
        if let Some(snapshot) = self.redo_stack.pop_back() {
            // Save current state to undo stack
            self.undo_stack.push_back(StateSnapshot {
                canvas: current,
                action: snapshot.action.clone(),
            });
            Some(snapshot.canvas)
        } else {
            None
        }
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    /// Get undo action description
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|s| s.action.as_str())
    }
    
    /// Get redo action description
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|s| s.action.as_str())
    }
    
    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

/// Action types for history
#[derive(Debug, Clone)]
pub enum HistoryAction {
    /// Block was added
    AddBlock { block_id: String },
    /// Block was removed
    RemoveBlock { block_id: String, block_name: String },
    /// Block was moved
    MoveBlock { block_id: String },
    /// Connection was created
    AddConnection { from: String, to: String },
    /// Connection was removed
    RemoveConnection { from: String, to: String },
    /// Multiple blocks selected
    SelectionChange,
    /// Canvas cleared
    ClearCanvas,
}

impl HistoryAction {
    pub fn description(&self) -> String {
        match self {
            HistoryAction::AddBlock { block_id } => format!("Add block {}", block_id),
            HistoryAction::RemoveBlock { block_name, .. } => format!("Remove {}", block_name),
            HistoryAction::MoveBlock { block_id } => format!("Move {}", block_id),
            HistoryAction::AddConnection { from, to } => format!("Connect {} → {}", from, to),
            HistoryAction::RemoveConnection { from, to } => format!("Disconnect {} → {}", from, to),
            HistoryAction::SelectionChange => "Change selection".to_string(),
            HistoryAction::ClearCanvas => "Clear canvas".to_string(),
        }
    }
}
