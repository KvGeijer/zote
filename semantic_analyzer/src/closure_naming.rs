use std::collections::HashMap;

use parser::{Expr, ExprNode, LValue, Stmts};

use crate::{ref_id, visitor::AstVisitor, NodeAttr, RefId};

/// Finds functions declared like
///     f := (__args__) -> ... ; or fn f(__args__) -> ... ;
/// and assigns them their name to use for recursive calls (here f).
///
/// This is because all function declarations are parsed to a binding of
/// a func def to a variable, where the func def does not know the variables
/// name. However, in case where is is bound directly to a variable, we want
/// to be able to call it recursively with the variable name (even if it can
/// be re-assigned later).
///
/// However, take the following example:
///     f := (n) -> if n > 0 f(n-8) else print(n);
///     f = (n) -> if n == 0 print("ZERO!") else f(0);
///     f(1) >> print;
/// What should this print?
/// In Zote, this will print `-7`, because the reference to f in the second
/// definition will not be recursive, as it is a re-assignment, and not a
/// declaration of f.
/// This is a bit of a corner case, but becomes important for our strange
/// recursive closure definitions.
pub(crate) fn find_recursion_names(ast: &Stmts) -> HashMap<RefId, NodeAttr> {
    let mut finder = NameFinder {
        bindings: HashMap::new(),
    };

    finder.visit_stmts(ast);

    finder
        .bindings
        .into_iter()
        .map(|(id, name)| (id, NodeAttr::RecursionName(name)))
        .collect()
}

struct NameFinder {
    /// Binds the id of Expr (of a func def), to a potential binding for recursion.
    bindings: HashMap<RefId, String>,
}

impl AstVisitor for NameFinder {
    fn visit_decl(&mut self, lvalue: &LValue, init: Option<&ExprNode>) {
        if let LValue::Var(name) = lvalue {
            if let Some(expr_node) = init {
                if matches!(expr_node.node.as_ref(), Expr::FunctionDefinition(_, _, _)) {
                    // Direct declaration of function, where we should support recursion
                    self.bindings
                        .insert(ref_id(expr_node.node.as_ref()), name.clone());
                }
            }
        }

        self.visit_lvalue(lvalue, true);
        if let Some(expr) = init {
            self.visit_expr(expr)
        }
    }
}
