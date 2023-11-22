use parser::{
    BinOper, CodeRange, Expr, ExprNode, Index, LValue, ListContent, LogicalOper, Slice, Stmt,
    StmtNode, Stmts, UnOper,
};

use super::{Chunk, CompRes, CompRetRes, Compiler, OpCode};
use crate::value::Value;

mod conditionals;
mod function;

impl Compiler<'_> {
    pub fn compile_statement(&mut self, statement: &StmtNode, chunk: &mut Chunk, output: bool) {
        let StmtNode {
            node,
            start_loc,
            end_loc,
        } = statement;
        let range = CodeRange::from_locs(*start_loc, *end_loc);

        let res = match node.as_ref() {
            Stmt::Decl(lvalue, expr) => {
                self.compile_declaration(lvalue, expr.as_ref(), range.clone(), chunk)
            }
            Stmt::Expr(expr) => {
                let res = self.compile_expression(expr, chunk);
                if !output {
                    // Keep the value if the statement should keep output
                    chunk.push_opcode(OpCode::Discard, range.clone());
                }
                res
            }
            Stmt::Invalid => panic!("Cannot interpret invalid statements!"),
        };

        if let Err(reason) = res {
            // TODO: Push some garbage opcode?
            eprintln!("COMPILER ERROR: [{range}] {reason}");
            self.had_error = true;
        }
    }

    fn compile_declaration(
        &mut self,
        lvalue: &LValue,
        expr: Option<&ExprNode>,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        if !self.is_global() {
            self.declare_local(lvalue, range.clone(), chunk, true)?;
        }
        if let Some(expr) = expr {
            self.compile_lvalue_assignment(lvalue, expr, range.clone(), chunk)?;
            chunk.push_opcode(OpCode::Discard, range);
        }
        Ok(())
    }

    /// Declares a local
    ///
    /// Mostly no codegen, but it assigns pointers to NIL for declared pointers.
    /// lvalue_top specifies if this is the topmost lvalue in a declaration
    fn declare_local(
        &mut self,
        lvalue: &LValue,
        range: CodeRange,
        chunk: &mut Chunk,
        lvalue_top: bool,
    ) -> CompRes {
        match lvalue {
            LValue::Index(_, _) => {
                return Err(format!("Cannot assign at an index in a declaration"))
            }
            LValue::Var(name) => {
                if self.attributes.is_upvalue(name) {
                    // Declares the local as a pointer insteal of a flat value
                    let offset = self.locals.add_local(name.to_owned(), true);

                    // TODO: THis could be done with eg semantic analysis help in the first assignment
                    // which would save computation, and remove codegen from this function.

                    // Assign it a new empty pointer
                    chunk.push_opcode(OpCode::EmptyPointer, range.clone());
                    chunk.push_opcode(OpCode::AssignLocal, range.clone());
                    chunk.push_u8_offset(offset);
                } else {
                    self.locals.add_local(name.to_owned(), false);
                }
            }
            LValue::Tuple(lvalues) => {
                for lvalue in lvalues.iter() {
                    self.declare_local(lvalue, range.clone(), chunk, false)?;
                }
            }
            LValue::Constant(_) if lvalue_top => return Err(format!("Cannot declare a constant")),
            LValue::Constant(_) => (),
        }
        Ok(())
    }

    pub fn declare_global(&mut self, name: &str) -> usize {
        let len = self.globals.len();
        *self.globals.entry(name.to_string()).or_insert(len)
    }

    // TODO: Variable resolution
    fn compile_expression(&mut self, expr: &ExprNode, chunk: &mut Chunk) -> CompRes {
        let ExprNode {
            node,
            start_loc,
            end_loc,
        } = expr;
        let range = CodeRange::from_locs(*start_loc, *end_loc);

        match node.as_ref() {
            Expr::Call(func, args) => self.compile_call(func, args, range, chunk)?,
            Expr::IndexInto(base, index) => self.compile_index_into(base, index, range, chunk)?,
            Expr::Binary(x, binop, y) => {
                self.compile_expression(x, chunk)?;
                self.compile_expression(y, chunk)?;
                let opcode = binop_opcode_conv(binop);
                chunk.push_opcode(opcode, range);
            }
            Expr::Unary(unop, x) => {
                self.compile_expression(x, chunk)?;
                let opcode = unop_opcode_conv(unop);
                chunk.push_opcode(opcode, range);
            }
            Expr::Logical(lhs, LogicalOper::And, rhs) => {
                self.compile_and(lhs, rhs, range, chunk)?
            }
            Expr::Logical(lhs, LogicalOper::Or, rhs) => self.compile_or(lhs, rhs, range, chunk)?,
            Expr::Assign(lvalue, expr) => {
                self.compile_lvalue_assignment(lvalue, expr, range, chunk)?;
            }
            Expr::Var(name) => self.compile_var(name, range, chunk)?,
            Expr::Int(x) => chunk.push_constant_plus(Value::Int(*x), range),
            Expr::Float(x) => chunk.push_constant_plus(Value::Float(*x), range),
            Expr::Bool(x) => chunk.push_constant_plus(Value::Bool(*x), range),
            Expr::String(string) => {
                chunk.push_constant_plus((string.as_ref() as &str).into(), range)
            }
            Expr::Block(stmts) => self.compile_block(stmts, range, chunk),
            Expr::If(pred, then, otherwise) => {
                self.compile_if(pred, then, otherwise.as_ref(), range, chunk)?
            }
            Expr::While(pred, body) => self.compile_while(pred, body, range, chunk)?,
            Expr::For(lvalue, collection, body) => {
                self.compile_for(lvalue, collection, body, range, chunk)?
            }
            Expr::Break => self.compile_break(range, chunk)?,
            Expr::Continue => self.compile_continue(range, chunk)?,
            Expr::Return(opt_expr) => self.compile_return(opt_expr.as_ref(), range, chunk)?,
            Expr::Nil => chunk.push_constant_plus(Value::Nil, range),
            Expr::List(list) => self.compile_list(list, range, chunk)?,
            Expr::Tuple(_) => {
                return Err("Tuples not implemented as expressions. Use a list.".to_owned())
            }
            Expr::FunctionDefinition(name, params, body) => {
                let upvalues = self.attributes.upvalue_names(node.as_ref()).unwrap_or(&[]);
                let rec_name = self.attributes.rec_name(node.as_ref());
                let nbr_locals = self
                    .attributes
                    .local_count(node.as_ref())
                    .expect("Function must have local count"); // TODO: DO this in compilation phase?

                self.compile_function_def(
                    name, rec_name, params, body, upvalues, nbr_locals, range, chunk,
                )?;
            }
            Expr::Match(base, arms) => self.compile_match(base, arms, range, chunk)?,
        };

        Ok(())
    }

    fn compile_lvalue_assignment(
        &mut self,
        lvalue: &LValue,
        expr: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        self.compile_expression(expr, chunk)?;

        chunk.push_opcode(OpCode::Duplicate, range.clone());

        self.compile_assign(lvalue, range, chunk)
    }

    /// Assigns the top value on the temp stack to the lvalue
    /// Consumes the assigned value.
    fn compile_assign(&mut self, lvalue: &LValue, range: CodeRange, chunk: &mut Chunk) -> CompRes {
        match lvalue {
            LValue::Index(collection, index) => {
                self.compile_assign_index(collection, index, range, chunk)
            }
            LValue::Var(name) => self.compile_assign_var(name, range, chunk),
            LValue::Tuple(lvalues) => self.compile_assign_tuple(lvalues, range, chunk),
            LValue::Constant(expected) => self.compile_assign_constant(expected, range, chunk),
        }
    }

    /// Compiles the assignment into a tuple of values
    fn compile_assign_tuple(
        &mut self,
        lvalues: &[LValue],
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        chunk.push_opcode(OpCode::TopToIter, range.clone());

        for (ind, lvalue) in lvalues.iter().enumerate() {
            // Clone the iterable as we want to use it several times and it will be consumed in ReadAtIndex
            chunk.push_opcode(OpCode::Duplicate, range.clone());

            // Push the indexed value
            chunk.push_constant_plus((ind as i64).into(), range.clone());
            chunk.push_opcode(OpCode::ReadAtIndex, range.clone());
            // TODO: Better error handling?

            // Assign it to the lvalue
            self.compile_assign(lvalue, range.clone(), chunk)?;
        }

        // Check that there are no more values in the iterable
        chunk.push_constant_plus((lvalues.len() as i64).into(), range.clone());
        chunk.push_opcode(OpCode::NextOrJump, range.clone());
        let ok_exit = chunk.reserve_jump();

        chunk.push_constant_plus(
            "Too many values to unpack in tuple assignment".into(),
            range.clone(),
        );
        chunk.push_opcode(OpCode::RaiseError, range.clone());

        // Discard the index and RHS
        chunk.patch_reserved_jump(ok_exit);
        chunk.push_opcode(OpCode::Discard, range.clone());
        chunk.push_opcode(OpCode::Discard, range);

        Ok(())
    }

    /// Compiles code to assert the expression is equal to the constant
    fn compile_assign_constant(
        &mut self,
        expected: &ExprNode,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        self.compile_expression(expected, chunk)?;
        chunk.push_opcode(OpCode::NonEquality, range.clone());

        chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
        let reserved_ok = chunk.reserve_jump();

        chunk.push_constant_plus(
            "Assignment to constant failed (not equal)".into(),
            range.clone(),
        );
        chunk.push_opcode(OpCode::RaiseError, range.clone());

        chunk.patch_reserved_jump(reserved_ok);

        Ok(())
    }

    /// Compiles code to assign the top-most temp stack value into an indexed value such as list[index]
    fn compile_assign_index(
        &mut self,
        collection: &ExprNode,
        index: &Index,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        match index {
            Index::At(expr_at) => {
                self.compile_expression(collection, chunk)?;
                self.compile_expression(expr_at, chunk)?;
                chunk.push_opcode(OpCode::AssignAtIndex, range);
            }
            Index::Slice(slice) => {
                // First convert the rhs to an iterator
                // TOOD: Be able to assign all of a slice to a single value
                chunk.push_opcode(OpCode::TopToIter, range.clone());
                self.compile_expression(collection, chunk)?;
                self.compile_assign_slice(slice, range, chunk)?;
            }
        }

        Ok(())
    }

    /// For assigning into a slice of a value
    fn compile_assign_slice(
        &mut self,
        slice: &Slice,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        if let Some(stop) = &slice.stop {
            // Compile in the correct order for side effects
            match &slice.start {
                Some(expr) => self.compile_expression(expr, chunk)?,
                None => chunk.push_constant_plus(Value::Int(0), range.clone()),
            };
            self.compile_expression(stop, chunk)?;
        } else {
            // Swap order of compilation to get access to list len
            chunk.push_opcode(OpCode::Duplicate, range.clone());
            chunk.push_opcode(OpCode::Len, range.clone());
            match &slice.start {
                Some(expr) => self.compile_expression(expr, chunk)?,
                None => chunk.push_constant_plus(Value::Int(0), range.clone()),
            };
            chunk.push_opcode(OpCode::Swap, range.clone());
        }

        self.compile_opt_expression(slice.step.as_ref(), chunk)?;
        chunk.push_opcode(OpCode::ListFromSlice, range.clone());

        // To set up the loop index
        chunk.push_constant_plus(Value::Int(0), range.clone());

        // Start assigning into the sliced value
        self.compile_assign_between_iterables(range, chunk)?;

        Ok(())
    }

    /// Assigns into a subset of an iterable from another iterable
    ///
    /// For repeatedly computing `Assignee[Slice[Index]] <- RHS[Index]`
    /// Then doing this until Index goes out of bounds for Slice, after which it checks if RHS is exhausted
    /// Stack state:
    ///    - Index
    ///    - Slice
    ///    - Assignee
    ///    - RHS
    fn compile_assign_between_iterables(&mut self, range: CodeRange, chunk: &mut Chunk) -> CompRes {
        let start_label = chunk.len();

        // 1: Calculate the next value from Slice. Potentially completing the assignment loop
        chunk.push_opcode(OpCode::NextOrJump, range.clone());
        let reserved_exit = chunk.reserve_jump();

        // 2: Monster instruction to assign between RHS and Assignee
        chunk.push_opcode(OpCode::AssignSliceIndex, range.clone());

        // 3: Jump back to continue with the next index
        chunk.push_opcode(OpCode::Jump, range.clone());
        chunk.push_jump(start_label);

        // Exit: When the slice have ran out, jump here
        chunk.patch_reserved_jump(reserved_exit);

        // Discard Slice and Assignee
        // TODO: Efficiency
        chunk.push_opcode(OpCode::Swap, range.clone());
        chunk.push_opcode(OpCode::Discard, range.clone());
        chunk.push_opcode(OpCode::Swap, range.clone());
        chunk.push_opcode(OpCode::Discard, range.clone());

        // Exit check: Make sure that we have run out of values in RHS (TODO: Should we do this earlier with tools from match statements?)
        chunk.push_opcode(OpCode::NextOrJump, range.clone()); // we want this to fail
        let ok_exit = chunk.reserve_jump();

        // Value will be on stack, but we don't care as we crash

        // ERROR! Can't match!
        chunk.push_constant_plus(
            "MATCH ERROR: The RHS value is of larger dimension than the assignee".into(),
            range.clone(),
        );
        chunk.push_opcode(OpCode::RaiseError, range.clone());

        // TODO: Do this check in the beginning, so that we can actually print the lengths?
        // Exit the match successfully
        chunk.patch_reserved_jump(ok_exit);

        // Get rid of index and RHS
        chunk.push_opcode(OpCode::Discard, range.clone());
        chunk.push_opcode(OpCode::Discard, range.clone());

        Ok(())
    }

    /// Assigns the topmost temp value to the named variable
    fn compile_assign_var(&mut self, name: &str, range: CodeRange, chunk: &mut Chunk) -> CompRes {
        if let Some((offset, pointer)) = self.locals.get_local(name) {
            if !pointer {
                chunk.push_opcode(OpCode::AssignLocal, range);
            } else {
                chunk.push_opcode(OpCode::AssignPointer, range);
            }
            chunk.push_u8_offset(offset);
            Ok(())
        } else if let Some(offset) = self.locals.get_upvalue(name) {
            chunk.push_opcode(OpCode::AssignUpValue, range);
            chunk.push_u8_offset(offset as u8);
            Ok(())
        } else if let Some(&offset) = self.globals.get(name) {
            chunk.push_opcode(OpCode::AssignGlobal, range); // Maybe bad range choice
            chunk.push_u8_offset(offset as u8);
            Ok(())
        } else {
            Err(format!("Global var '{name}' is not declared"))
        }
    }

    /// Compiles the expression, or just push Nil (without location) if no expression
    fn compile_opt_expression(&mut self, expr: Option<&ExprNode>, chunk: &mut Chunk) -> CompRes {
        // Maybe we should be smarter and never push such a Nil value
        match expr {
            Some(expr) => self.compile_expression(expr, chunk)?,
            None => chunk.push_constant_plus(Value::Nil, CodeRange::from_ints(0, 0, 0, 0, 0, 0)),
        };
        Ok(())
    }

    /// Compiles the read of a var.
    fn compile_var(&mut self, name: &str, range: CodeRange, chunk: &mut Chunk) -> CompRes {
        if let Some((offset, pointer)) = self.locals.get_local(name) {
            if !pointer {
                chunk.push_opcode(OpCode::ReadLocal, range);
            } else {
                chunk.push_opcode(OpCode::ReadPointer, range);
            }
            chunk.push_u8_offset(offset);
            Ok(())
        } else if let Some(offset) = self.locals.get_upvalue(name) {
            chunk.push_opcode(OpCode::ReadUpValue, range);
            chunk.push_u8_offset(offset as u8);
            Ok(())
        } else if let Some(offset) = self.globals.get(name) {
            chunk.push_opcode(OpCode::ReadGlobal, range);
            chunk.push_u8_offset(*offset as u8);
            Ok(())
        } else {
            // ERROR: Compile error!
            Err(format!("Var '{name}' is not declared"))
        }
    }

    /// Declare all top-level declarations, so as to use late-binding
    pub fn declare_globals(&mut self, stmts: &Stmts) {
        for stmt in stmts.stmts.iter() {
            if let Stmt::Decl(lvalue, _) = stmt.node.as_ref() {
                self.declare_global_lvalue(lvalue);
            }
        }
    }

    fn declare_global_lvalue(&mut self, lvalue: &LValue) {
        match lvalue {
            LValue::Index(_, _) => (),
            LValue::Var(name) => {
                self.declare_global(name);
            }
            LValue::Tuple(lvalues) => {
                for lvalue in lvalues.iter() {
                    self.declare_global_lvalue(lvalue);
                }
            }
            LValue::Constant(_) => (),
        }
    }

    /// Compiles the block of statements. Does not throw, as errors are printed and escaped within.
    fn compile_block(&mut self, stmts: &Stmts, range: CodeRange, chunk: &mut Chunk) {
        self.locals.enter();
        self.compile_stmts(stmts, chunk);
        if !stmts.output || stmts.stmts.is_empty() {
            chunk.push_opcode(OpCode::Nil, range.clone());
        }

        let pointer_offsets = self.locals.exit();
        self.drop_pointers(&pointer_offsets, range, chunk);
    }

    /// Explicitly drops pointers at the specified offsets from rbp
    fn drop_pointers(&mut self, offsets: &[u8], range: CodeRange, chunk: &mut Chunk) {
        for &offset in offsets {
            chunk.push_opcode(OpCode::Drop, range.clone());
            chunk.push_u8_offset(offset);
        }
    }

    /// Compiles a list constant
    fn compile_list(&mut self, list: &ListContent, range: CodeRange, chunk: &mut Chunk) -> CompRes {
        match list {
            ListContent::Exprs(exprs) => {
                if exprs.len() > 255 {
                    // As we store the length in a byte we cannot store too many
                    return Err(format!(
                        "Cannot init list with over 255 values :( This one is {} long",
                        exprs.len()
                    ));
                }

                for expr in exprs {
                    self.compile_expression(expr, chunk)?;
                }
                chunk.push_opcode(OpCode::ListFromValues, range);
                chunk.push_u8_offset(exprs.len() as u8);
            }
            ListContent::Range(slice) => {
                self.compile_slice(slice, chunk)?;
                chunk.push_opcode(OpCode::ListFromSlice, range);
            }
        }
        Ok(())
    }

    /// Compiles computations for the three parts of the slice
    ///
    /// If any of the fields are omitted, a NIL is pushed instead
    fn compile_slice(&mut self, slice: &Slice, chunk: &mut Chunk) -> CompRes {
        self.compile_opt_expression(slice.start.as_ref(), chunk)?;
        self.compile_opt_expression(slice.stop.as_ref(), chunk)?;
        self.compile_opt_expression(slice.step.as_ref(), chunk)
    }

    /// Compiles the indexing into a list
    fn compile_index_into(
        &mut self,
        base: &ExprNode,
        index: &Index,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        self.compile_expression(base, chunk)?;
        match index {
            Index::At(at) => {
                self.compile_expression(at, chunk)?;
                chunk.push_opcode(OpCode::ReadAtIndex, range);
            }
            Index::Slice(slice) => {
                self.compile_slice(slice, chunk)?;
                chunk.push_opcode(OpCode::ReadAtSlice, range);
            }
        }
        Ok(())
    }

    // Try to match and assign to every pattern in a row, otherwise crashing
    fn compile_match(
        &mut self,
        base: &ExprNode,
        patterns: &[(LValue, ExprNode)],
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRes {
        // First, compile the base expression
        self.compile_expression(base, chunk)?;

        // All reserved jumps for exiting the match expressions successfully
        let mut reserved_exit_jumps = vec![];

        for (pattern, then) in patterns.iter() {
            // Enter the match arm scope
            self.locals.enter();

            // Try to match against the pattern
            let reserved_match_fail_jumps =
                self.compile_try_match(pattern, range.clone(), chunk)?;

            // If succesfull, assign into the pattern, consuming the value
            self.declare_local(pattern, range.clone(), chunk, true)?;
            self.compile_assign(pattern, range.clone(), chunk)?;

            // Execute the expression, and leave it as the top stack value
            self.compile_expression(then, chunk)?;

            // Exit the match arm scope
            let pointer_offsets = self.locals.exit();
            self.drop_pointers(&pointer_offsets, range.clone(), chunk);

            // Finally, exit the whole match expression
            chunk.push_opcode(OpCode::Jump, range.clone());
            reserved_exit_jumps.push(chunk.reserve_jump());

            // If the match fails, try the next pattern
            for reserved in reserved_match_fail_jumps {
                chunk.patch_reserved_jump(reserved);
            }
        }

        // No pattern matched, so error
        chunk.push_constant_plus(
            "Exhausted match patterns. No possible match found.".into(),
            range.clone(),
        );
        chunk.push_opcode(OpCode::RaiseError, range.clone());

        // Path the exit jumps
        for reserved in reserved_exit_jumps {
            chunk.patch_reserved_jump(reserved);
        }

        Ok(())
    }

    /// Try to match the top of the stack against an lvalue
    ///
    /// Returns all reserved indecies where the match failed and it should jump to the next pattern
    /// Does not consume the top stack value. TODO: Should we consume it?
    /// Does not actually assign or do anything in a succesful case.
    fn compile_try_match(
        &mut self,
        pattern: &LValue,
        range: CodeRange,
        chunk: &mut Chunk,
    ) -> CompRetRes<Vec<usize>> {
        let mut abort_jumps = vec![];

        match pattern {
            LValue::Index(_, _) => {
                // TODO: This could be supported as a normal variable binding if we want
                return Err("Index-into lvalues not supported in match expressions".to_owned());
            }
            LValue::Var(_) => (), // Can match against anything
            LValue::Tuple(lvalues) => {
                // Check that the length is ok
                chunk.push_opcode(OpCode::Duplicate, range.clone());
                chunk.push_opcode(OpCode::Len, range.clone());
                chunk.push_constant_plus((lvalues.len() as i64).into(), range.clone());
                chunk.push_opcode(OpCode::Equality, range.clone());
                chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
                abort_jumps.push(chunk.reserve_jump());

                // Then check that it can match against all individual values
                for (ind, lvalue) in lvalues.iter().enumerate() {
                    if let LValue::Var(_) = lvalue {
                        // Don't have to check anything here, just for simplicity
                        continue;
                    }
                    // Take out the value from the collection
                    chunk.push_opcode(OpCode::Duplicate, range.clone());
                    chunk.push_constant_plus((ind as i64).into(), range.clone());
                    chunk.push_opcode(OpCode::ReadAtIndex, range.clone());

                    // See if the indexed value matches the lvalue
                    abort_jumps.extend_from_slice(&mut self.compile_try_match(
                        lvalue,
                        range.clone(),
                        chunk,
                    )?);

                    // Remember to discard the indexed value
                    chunk.push_opcode(OpCode::Discard, range.clone());
                }
            }
            LValue::Constant(constant) => {
                chunk.push_opcode(OpCode::Duplicate, range.clone());
                self.compile_expression(constant, chunk)?;
                chunk.push_opcode(OpCode::Equality, range.clone());

                // If the value is not equal to the constant, abort match
                chunk.push_opcode(OpCode::JumpIfFalse, range.clone());
                abort_jumps.push(chunk.reserve_jump());
            }
        }

        Ok(abort_jumps)
    }
}

fn binop_opcode_conv(binop: &BinOper) -> OpCode {
    match binop {
        BinOper::Add => OpCode::Add,
        BinOper::Sub => OpCode::Subtract,
        BinOper::Div => OpCode::Divide,
        BinOper::Mult => OpCode::Multiply,
        BinOper::Mod => OpCode::Modulo,
        BinOper::Pow => OpCode::Power,
        BinOper::Eq => OpCode::Equality,
        BinOper::Neq => OpCode::NonEquality,
        BinOper::Lt => OpCode::LessThan,
        BinOper::Leq => OpCode::LessEqual,
        BinOper::Gt => OpCode::GreaterThan,
        BinOper::Geq => OpCode::GreaterEqual,
        BinOper::Append => OpCode::Append,
    }
}

fn unop_opcode_conv(unop: &UnOper) -> OpCode {
    match unop {
        UnOper::Not => OpCode::Not,
        UnOper::Sub => OpCode::Negate,
    }
}
