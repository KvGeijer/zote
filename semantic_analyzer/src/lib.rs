use std::collections::HashMap;

use parser::Stmts;

mod visitor;

pub struct AttributedAst<'a> {
    stmts: &'a Stmts,
    attributes: HashMap<usize, Vec<NodeAttr>>,
}

fn ref_id<T>(reference: &T) -> usize {
    let raw_pointer = reference as *const T;
    raw_pointer as usize
}

enum NodeAttr {}
