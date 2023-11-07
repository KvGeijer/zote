use std::collections::HashMap;

use parser::Stmts;
use variable_resolution::find_upvalues;

mod variable_resolution;
mod visitor;

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

    fn new(stmts: &'a Stmts) -> Self {
        Self {
            stmts,
            attributes: HashMap::new(),
        }
    }

    fn merge_singles(&mut self, attrs: HashMap<RefId, NodeAttr>) {
        for (id, attr) in attrs {
            println!("Found upvalue");
            self.attributes.entry(id).or_insert(vec![]).push(attr)
        }
    }

    /// Finds which declarations are upvalues
    fn find_upvalues(&mut self) {
        self.merge_singles(find_upvalues(self.stmts));
    }
}

#[derive(PartialEq)]
enum NodeAttr {
    UpValue,
}
