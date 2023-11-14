use std::collections::HashMap;

use closure_naming::find_recursion_names;
use local_enumerator::count_locals;
use parser::{Expr, Stmts};
use variable_resolution::find_upvalues;

mod closure_naming;
mod local_enumerator;
mod variable_resolution;
mod visitor;

#[derive(Debug)]
pub struct AttributedAst<'a> {
    stmts: &'a Stmts,
    attributes: HashMap<usize, Vec<NodeAttr>>,
}

pub fn analyze_ast<'a>(stmts: &'a Stmts) -> AttributedAst<'a> {
    let mut attr_ast = AttributedAst::new(stmts);
    attr_ast.analyze_variable_bindings();
    attr_ast.merge_singles(count_locals(stmts));

    attr_ast
}

type RefId = usize;

fn ref_id<T>(reference: &T) -> RefId {
    let raw_pointer = reference as *const T;
    raw_pointer as usize
}

impl<'a> AttributedAst<'a> {
    /// Used on the reference to the name of a variable declaration to see if it is an upvalue
    pub fn is_upvalue(&self, var_ref: &String) -> bool {
        if let Some(attrs) = self.attributes.get(&ref_id(var_ref)) {
            attrs.contains(&NodeAttr::UpValue)
        } else {
            false
        }
    }

    /// Returns all upvalue names for a function definition Expression.
    /// Borrows the attributes for as long as the names live.
    pub fn upvalue_names<'b>(&'b self, func_ref: &Expr) -> Option<&'b [String]> {
        if !matches!(func_ref, Expr::FunctionDefinition(_, _, _)) {
            // Now we see why this approach is very bad with loose pointers
            panic!("Cannot call upvalue_names on a non-function def Expr")
        }

        if let Some(attrs) = self.attributes.get(&ref_id(func_ref)) {
            attrs.iter().find_map(|attr| {
                if let NodeAttr::UpValues(names) = attr {
                    Some(names.as_slice())
                } else {
                    None
                }
            })
        } else {
            None
        }
    }

    /// For a FunctionDefinition Expr, returns the potential name to use for recursive calls
    pub fn rec_name(&self, func_ref: &Expr) -> Option<String> {
        self.recursion_name_raw(ref_id(func_ref))
    }

    /// Get the max number of locals for a function definition, including arguments
    pub fn local_count(&self, func_ref: &Expr) -> Option<usize> {
        let id = ref_id(func_ref);
        self.local_count_raw(id)
    }

    /// Gets the max number of locals in in the script, outside all functions
    pub fn global_local_count(&self) -> usize {
        let id = ref_id(self.stmts);
        self.local_count_raw(id)
            .expect("Should have a scipt-level local count")
    }

    pub fn local_count_raw(&self, id: RefId) -> Option<usize> {
        self.attributes.get(&id)?.into_iter().find_map(|attr| {
            if let NodeAttr::LocalCount(count) = attr {
                Some(*count)
            } else {
                None
            }
        })
    }

    fn recursion_name_raw(&self, id: RefId) -> Option<String> {
        self.attributes.get(&id)?.into_iter().find_map(|attr| {
            if let NodeAttr::RecursionName(name) = attr {
                Some(name.clone())
            } else {
                None
            }
        })
    }

    pub fn stmts(&self) -> &'a Stmts {
        self.stmts
    }

    pub fn new(stmts: &'a Stmts) -> Self {
        Self {
            stmts,
            attributes: HashMap::new(),
        }
    }

    fn merge_singles(&mut self, attrs: HashMap<RefId, NodeAttr>) {
        for (id, attr) in attrs {
            self.attributes.entry(id).or_insert(vec![]).push(attr)
        }
    }

    /// Finds:
    ///    * Names for functions when recursing
    ///    * Which declarations are upvalues
    ///    * Which upvalues are included in each closure init
    fn analyze_variable_bindings(&mut self) {
        self.merge_singles(find_recursion_names(self.stmts));

        // Requires the names of recursive functions to work properly
        let upvalue_attrs = find_upvalues(self.stmts, self);
        self.merge_singles(upvalue_attrs);
    }
}

#[derive(Debug, PartialEq)]
enum NodeAttr {
    /// If a declaration is an upvalue
    UpValue,

    /// All upvalues captured by a function
    UpValues(Vec<String>),

    /// Name binding for a FunctionDefinition to use when recursing
    RecursionName(String),

    /// The max number of locals declared in each function definition
    LocalCount(usize),
}
