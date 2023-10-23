pub struct LocalState {
    locals: Vec<Local>,
    scope_depth: u8,
}

impl LocalState {
    pub fn new() -> Self {
        Self {
            locals: Vec::with_capacity(256),
            scope_depth: 0,
        }
    }

    /// Is the compiler currently in global scope?
    pub fn is_global(&self) -> bool {
        // TODO: Fix when nesting
        self.scope_depth == 0
    }

    /// Returns the offset of the local variable from the rbp
    pub fn get(&self, var: &str) -> Option<u8> {
        for (ind, local) in self.locals.iter().enumerate().rev() {
            if local.name.len() == var.len() && local.name == var {
                return Some(ind as u8);
            }
        }
        None
    }

    pub fn add_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        })
    }

    /// Enter a new local scope
    pub fn enter(&mut self) {
        self.scope_depth += 1
    }

    /// Exit a local scope
    pub fn exit(&mut self) {
        while !self.locals.is_empty() && self.locals.last().unwrap().depth == self.scope_depth {
            self.locals.pop();
        }

        self.scope_depth -= 1
    }
}

struct Local {
    name: String,
    depth: u8,
}
