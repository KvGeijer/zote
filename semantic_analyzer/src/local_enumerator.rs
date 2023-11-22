use std::{collections::HashMap, mem};

use parser::{ExprNode, LValue, Stmts};

use crate::{ref_id, visitor::AstVisitor, NodeAttr, RefId};

/// Counts how many locals there are at most in each function (including arguments)
/// This only counts args and locals, no functions or other things on the stack
pub(crate) fn count_locals(stmts: &Stmts) -> HashMap<RefId, NodeAttr> {
    // TODO: We could just do this as part of the compilation pass.
    let mut counter = Counter {
        scope: VarScope {
            vars: vec![],
            parent: None,
            max_vars: 0,
            curr_depth: 0,
        },
        expr_id: None,
        global_scope: true,
        func_count: HashMap::new(),
    };

    counter.visit_stmts(stmts);

    // Also insert for the local scope
    counter
        .func_count
        .insert(ref_id(stmts), counter.scope.max_vars);

    counter
        .func_count
        .into_iter()
        .map(|(id, count)| (id, NodeAttr::LocalCount(count)))
        .collect()
}

struct Counter {
    scope: VarScope,

    /// The id of the most recently entered expression
    expr_id: Option<RefId>,

    /// If we are at global scope, meaning variables are globals
    global_scope: bool,

    /// The local + arg count for each function definition
    func_count: HashMap<RefId, usize>,
}

struct VarScope {
    /// One entry with the lexical depth of each variable in the function
    vars: Vec<usize>,

    /// The largest number of variables at any time in the given function
    max_vars: usize,

    /// The current lexical depth
    curr_depth: usize,

    /// The parent function scope
    parent: Option<Box<VarScope>>,
}

impl VarScope {
    /// Inserts a local
    fn insert(&mut self) {
        self.vars.push(self.curr_depth);
        self.max_vars = self.vars.len().max(self.max_vars);
    }

    fn enter_block(&mut self) {
        self.curr_depth += 1
    }

    /// Deallocates all locals in the block
    fn exit_block(&mut self) {
        while !self.vars.is_empty() && self.vars.last().unwrap() == &self.curr_depth {
            self.vars.pop();
        }
        self.curr_depth -= 1
    }

    fn nest(self) -> Self {
        Self {
            vars: vec![],
            parent: Some(Box::new(self)),
            max_vars: 0,
            curr_depth: 0,
        }
    }

    fn exit(self) -> Option<Self> {
        Some(*self.parent?)
    }

    fn empty() -> Self {
        Self {
            vars: vec![],
            max_vars: 0,
            curr_depth: 0,
            parent: None,
        }
    }
}

impl AstVisitor for Counter {
    fn visit_var(&mut self, _name: &String, declaration: bool) {
        if declaration {
            // Declare it as reachable
            self.scope.insert();
        }
    }

    fn visit_decl(&mut self, lvalue: &LValue, init: Option<&parser::ExprNode>) {
        if let Some(expr) = init {
            self.visit_expr(expr)
        }
        // Switch order to not define variables until after their values are defined
        self.visit_lvalue(lvalue, true);
    }

    fn visit_expr(&mut self, expr: &parser::ExprNode) {
        let outer_id = self.expr_id;
        self.expr_id = Some(ref_id(expr.node.as_ref()));

        // Default
        self.visit_expr_delegation(expr);

        self.expr_id = outer_id;
    }

    fn visit_function_definition(
        &mut self,
        _name: &str, // This is not the actual variable name, just a descriptive one
        params: &[LValue],
        body: &parser::ExprNode,
    ) {
        let id = self.expr_id.expect("Func def should be in expression");

        // Ugly way to do it
        let scope = mem::replace(&mut self.scope, VarScope::empty());
        self.scope = scope.nest();

        // Default visit
        for param in params {
            // Declares the params
            self.visit_lvalue(param, true);
        }
        self.visit_expr(body);
        // Default end

        // Store the actual result
        self.func_count.insert(id, self.scope.max_vars);

        // Ugly way to exit it
        let scope = mem::replace(&mut self.scope, VarScope::empty());
        self.scope = scope.exit().unwrap();
    }

    fn visit_block(&mut self, stmts: &Stmts) {
        let scope = self.global_scope;
        self.global_scope = false;

        // Default
        self.scope.enter_block();
        self.visit_stmts(stmts);
        self.scope.exit_block();

        self.global_scope = scope;
    }

    fn visit_match(&mut self, matched: &ExprNode, options: &[(LValue, ExprNode)]) {
        self.visit_expr(matched);
        for (lvalue, then) in options {
            self.scope.enter_block();
            self.visit_lvalue(lvalue, true);
            self.visit_expr(then);
            self.scope.exit_block();
        }
    }
}
