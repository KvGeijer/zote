use std::{
    collections::{HashMap, HashSet},
    mem,
};

use parser::{LValue, Stmts};

use crate::{ref_id, visitor::AstVisitor, NodeAttr, RefId};

pub(crate) fn find_upvalues(stmts: &Stmts) -> HashMap<RefId, NodeAttr> {
    let mut resolver = Resolver {
        scope: VarScope {
            vars: HashMap::new(),
            parent: None,
        },
        function_nesting: 0,
        upvalues: HashSet::new(),
    };

    resolver.visit_stmts(stmts);

    resolver
        .upvalues
        .into_iter()
        .map(|id| (id, NodeAttr::UpValue))
        .collect()
}

struct Resolver {
    scope: VarScope,

    // In how many nested function definitions are we?
    function_nesting: usize,

    /// Set of all Ids to Strings which are actually upvalues. A bit clunky as it really is the variable...
    /// Only for the declaration String, not its uses
    upvalues: HashSet<RefId>,
}

struct VarScope {
    /// Keeps both the reference id, as well as the function nesting it was declared at
    vars: HashMap<String, (RefId, usize)>,
    parent: Option<Box<VarScope>>,
}

impl VarScope {
    fn insert(&mut self, name: String, id: RefId, level: usize) {
        self.vars.insert(name, (id, level));
    }

    fn resolve(&self, name: &str) -> Option<(RefId, usize)> {
        if let Some(ret) = self.vars.get(name) {
            Some(*ret)
        } else if let Some(parent) = self.parent.as_ref() {
            parent.resolve(name)
        } else {
            None
        }
    }

    fn enter(self) -> Self {
        Self {
            vars: HashMap::new(),
            parent: Some(Box::new(self)),
        }
    }

    fn exit(self) -> Option<Self> {
        Some(*self.parent?)
    }

    fn empty() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }
}

impl AstVisitor for Resolver {
    fn visit_var(&mut self, name: &String, declaration: bool) {
        if declaration {
            self.scope
                .insert(name.to_string(), ref_id(name), self.function_nesting); // Should we use expr id in some way?
        } else if let Some((id, level)) = self.scope.resolve(name) {
            // Globals are also tagged as upvalues if captured, even they are treated differently
            if level != self.function_nesting {
                // It is an upvalue!
                self.upvalues.insert(id);
            }
        } else {
            // Could be a global which is forward declared, so ignore potential errors here
        }
    }

    fn visit_function_definition(
        &mut self,
        _name: &str, // This is not the actual variable name, just a descriptive one
        params: &[LValue],
        body: &parser::ExprNode,
    ) {
        self.function_nesting += 1;

        // Ugly way to do it
        // TODO: Good opportunity to play with unsafe?
        let scope = mem::replace(&mut self.scope, VarScope::empty());
        self.scope = scope.enter();

        // Default visit
        for param in params {
            self.visit_lvalue(param, true);
        }
        self.visit_expr(body);

        // Ugly way to do it
        let scope = mem::replace(&mut self.scope, VarScope::empty());
        self.scope = scope.exit().unwrap();

        self.function_nesting -= 1;
    }
}
