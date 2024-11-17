use crate::{AstNode, Expr, LValue, Stmt, Stmts};

/// Generate a string representing all globally declared functions
pub fn gen_functions_doc(stmts: &Stmts) -> String {
    let mut docs = String::new();

    for statement in stmts.stmts.iter() {
        match statement.node.as_ref() {
            Stmt::Decl(
                LValue::Var(func_name),
                Some(AstNode {
                    node: box Expr::FunctionDefinition(_, params, _),
                    start_loc: _,
                    end_loc: _,
                }),
            ) => {
                docs.push_str(&format!(
                    "fn {func_name}{};\n",
                    pretty_print_lvalues(params)
                ));
            }
            Stmt::Decl(_, _) => {}
            Stmt::Expr(_) => {}
            Stmt::Invalid => {}
        }
    }

    docs
}

/// Takes a vec of lvalues, and outputs the tuple representing them
fn pretty_print_lvalues(lvalues: &[LValue]) -> String {
    let mut doc = "(".to_string();

    for lvalue in lvalues.iter() {
        match lvalue {
            LValue::Var(name) => doc.push_str(name),
            LValue::Tuple(lvalues) => doc.push_str(&pretty_print_lvalues(lvalues)),

            // TODO: Make this better
            _ => panic!("Strange parameter enountered!"),
        }
        doc.push_str(", ");
    }

    // Pop the last ", ". Could do a much nicer solution, but will so rarely be called
    doc.pop();
    doc.pop();

    doc.push(')');
    doc
}
