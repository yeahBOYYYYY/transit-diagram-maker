use crate::graph::DocumentTree;

/// Manages undo and redo stacks for the document
#[derive(Debug, Clone)]
pub struct DocumentUndoHistory {
    undo_stack: Vec<DocumentTree>,
    redo_stack: Vec<DocumentTree>,
    max_history: usize,
}

impl DocumentUndoHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Records current document state before executing a modifying action
    pub fn push(&mut self, current_tree: DocumentTree) {
        self.undo_stack.push(current_tree);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current_tree: DocumentTree) -> Option<DocumentTree> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current_tree);
        Some(prev)
    }

    pub fn redo(&mut self, current_tree: DocumentTree) -> Option<DocumentTree> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current_tree);
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
