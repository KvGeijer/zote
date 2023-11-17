use std::{
    collections::{HashMap, HashSet},
    mem,
};

use parser::{LValue, Stmts};

use crate::{ref_id, visitor::AstVisitor, AttributedAst, NodeAttr, RefId};

/// Finds which declarations are upvalues
/// Also the upvalues (by name) are captured by each closure
pub(crate) fn find_upvalues(stmts: &Stmts, attributes: &AttributedAst) -> HashMap<RefId, NodeAttr> {
    let mut resolver = Resolver {
        scope: VarScope {
            vars: HashMap::new(),
            parent: None,
        },
        enclosing_functions: vec![],
        upvalues: HashSet::new(),
        closure_upvalues: HashMap::new(),
        expr_id: None,
        global_scope: true,
        attributes,
    };

    resolver.visit_stmts(stmts);

    // Finds which declarations are upvalues
    let upvalue_attrs = resolver.upvalues.iter().map(|&id| (id, NodeAttr::UpValue));

    // Finds which upvalues are captured by different closures
    let upvalues_attrs = resolver
        .closure_upvalues
        .into_iter()
        .map(|(id, names)| (id, NodeAttr::UpValues(names)));

    upvalue_attrs.chain(upvalues_attrs).collect()
}

struct Resolver<'a> {
    scope: VarScope,

    /// The id of all functions we are enclosed by at the moment
    /// This can be used to index into the closure_upvalues to update their attributes.
    enclosing_functions: Vec<RefId>,

    /// Set of all Ids to Strings which are actually upvalues. A bit clunky as it really is the variable...
    /// Only for the declaration String, not its uses
    upvalues: HashSet<RefId>,

    /// Maps each function def to how what upvalues it captures from the outer function
    /// Only contains their names. The offsets are calculated at compile time.
    closure_upvalues: HashMap<RefId, Vec<String>>,

    /// The id of the most recently entered expression
    expr_id: Option<RefId>,

    /// If we are at global scope, meaning variables are globals
    global_scope: bool,

    /// Map of function definition, to a potential name to use for recursion
    attributes: &'a AttributedAst<'a>,
}

impl<'a> Resolver<'a> {
    fn add_upvalue(&mut self, id: RefId, name: &String, func_level: usize) {
        // It is an upvalue!
        self.upvalues.insert(id);

        println!("Adding upvalue {name}");

        // Then we should also add it as an upvalue to all enclosing functions
        // where it was not declared.
        // ERROR: Off by one?
        for enclosing_func_id in self.enclosing_functions.iter().skip(func_level) {
            let func_upvalues = self
                .closure_upvalues
                .entry(*enclosing_func_id)
                .or_insert(vec![]);

            // Add the upvalue if it was not already added
            if !func_upvalues.contains(name) {
                func_upvalues.push(name.to_string())
            }
        }
    }
}

struct VarScope {
    /// Keeps both the reference id, as well as the function nesting it was declared at
    /// The id is of the reference to the
    /// Will only hold local variables (no globals!)
    /// For lexical nesting
    vars: HashMap<String, (RefId, usize)>,

    /// Parent for nesting of functions
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

impl<'a> AstVisitor for Resolver<'a> {
    fn visit_var(&mut self, name: &String, declaration: bool) {
        if declaration {
            // Declare it as reachable
            if !self.global_scope {
                self.scope.insert(
                    name.to_string(),
                    ref_id(name),
                    self.enclosing_functions.len(),
                );
            }
        } else if let Some((id, func_level)) = self.scope.resolve(name) {
            // globals are not tagged as upvalues
            if func_level != self.enclosing_functions.len() {
                println!("func_level {func_level}");
                self.add_upvalue(id, name, func_level);
            }
        } else {
            // Could be a global which is forward declared, so ignore potential errors here
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

        // Adds which enclosing functions exist
        self.enclosing_functions.push(id);

        // Ugly way to do it
        // TODO: Good opportunity to play with unsafe?
        let scope = mem::replace(&mut self.scope, VarScope::empty());
        self.scope = scope.enter();

        if !self.global_scope {
            // ERROR: What if we want to capture the closure as an upvalue? For now we don't allow that,
            // as that would mean we would have to handle the case where it is a pointer. As long
            // as it is 0 here, we will not support this.
            // If it has a recursive binding, add it to the scope
            if let Some(binding) = self.attributes.recursion_name_raw(id) {
                self.scope
                    .insert(binding.to_string(), 0, self.enclosing_functions.len());
            }
        }

        // Default visit
        for param in params {
            self.visit_lvalue(param, true);
        }
        self.visit_expr(body);

        // Ugly way to do it
        let scope = mem::replace(&mut self.scope, VarScope::empty());
        self.scope = scope.exit().unwrap();

        self.enclosing_functions.pop().unwrap();
    }

    fn visit_block(&mut self, stmts: &Stmts) {
        let scope = self.global_scope;
        self.global_scope = false;

        // Default
        self.visit_stmts(stmts);

        self.global_scope = scope;
    }
}
