use std::mem;

/// The state of declared variables, nested by closure definitions
pub struct LocalState {
    /// The parent in the stack of function scopes
    parent_locals: Option<Box<LocalState>>,

    /// The stack of currently defined locals. Both pointers and normal ones
    locals: Vec<Local>,

    /// The upvalues closed over from the outer function
    upvalues: Vec<UpValue>,

    /// The current lexical scope
    scope_depth: u8,
}

impl LocalState {
    pub fn new() -> Self {
        Self {
            locals: Vec::with_capacity(32),
            upvalues: Vec::with_capacity(8),
            scope_depth: 0,
            parent_locals: None,
        }
    }

    pub fn nest(&mut self) {
        let state = mem::replace(self, LocalState::new());
        self.parent_locals = Some(Box::new(state));
    }

    pub fn de_nest(&mut self) -> Option<()> {
        let state = mem::replace(self, LocalState::new());
        *self = *state.parent_locals?;
        // *self = *self.parent_locals?;
        Some(())
    }

    /// Is the compiler currently in global scope?
    pub fn is_global(&self) -> bool {
        self.scope_depth == 0 && self.parent_locals.is_none()
    }

    /// Returns the offset of the local variable from the rbp, as well as if it is a pointer
    pub fn get_local(&self, var: &str) -> Option<(u8, bool)> {
        for (ind, local) in self.locals.iter().enumerate().rev() {
            if local.name.len() == var.len() && local.name == var {
                return Some((ind as u8, local.pointer));
            }
        }
        None
    }

    /// Returns the index of the upvalue among the upvalues
    pub fn get_upvalue(&self, var: &str) -> Option<u8> {
        for (ind, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.name.len() == var.len() && upvalue.name == var {
                return Some(ind as u8);
            }
        }
        None
    }

    pub fn add_local(&mut self, name: String, pointer: bool) -> u8 {
        if self.locals.len() >= 255 {
            // TODO: Real error
            panic!("Cannot have more than 255 locals!");
        }

        self.locals.push(Local {
            name,
            depth: self.scope_depth,
            pointer,
        });
        (self.locals.len() - 1) as u8
    }

    pub fn add_upvalue(&mut self, name: String) {
        self.upvalues.push(UpValue { name })
    }

    /// Enter a new local scope
    pub fn enter(&mut self) {
        self.scope_depth += 1;
    }

    /// Exit a local scope
    ///
    /// Returns the offset of all pointers on the stack, which should be dropped.
    pub fn exit(&mut self) -> Vec<u8> {
        let mut pointers = vec![];
        while !self.locals.is_empty() && self.locals.last().unwrap().depth == self.scope_depth {
            if self.locals.pop().unwrap().pointer {
                pointers.push(self.locals.len() as u8)
            }
        }
        self.scope_depth -= 1;

        pointers
    }

    /// Returns the offset of all pointers on the stack.
    pub fn local_pointers(&mut self) -> Vec<u8> {
        self.locals
            .iter()
            .enumerate()
            .filter_map(|(offset, local)| local.pointer.then_some(offset as u8))
            .collect()
    }
}

struct Local {
    /// The same of the variable
    name: String,

    /// The nesting depth it is declared at
    depth: u8,

    /// If the variable is a pointer, so to be captured by closures
    pointer: bool,
}

struct UpValue {
    /// The same of the variable
    name: String,
}
