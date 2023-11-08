use std::collections::HashMap;

use parser::{Expr, Stmts};
use variable_resolution::find_upvalues;

mod variable_resolution;
mod visitor;

#[derive(Debug)]
pub struct AttributedAst<'a> {
    stmts: &'a Stmts,
    attributes: HashMap<usize, Vec<NodeAttr>>,
}

pub fn analyze_ast<'a>(stmts: &'a Stmts) -> AttributedAst<'a> {
    let mut attr_ast = AttributedAst::new(stmts);
    attr_ast.find_upvalues();

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

    /// Finds which declarations are upvalues
    fn find_upvalues(&mut self) {
        self.merge_singles(find_upvalues(self.stmts));
    }
}

#[derive(Debug, PartialEq)]
enum NodeAttr {
    /// If a declaration is an upvalue
    UpValue,

    /// All upvalues captured by a function
    UpValues(Vec<String>),
}
