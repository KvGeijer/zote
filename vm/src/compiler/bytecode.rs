use enum_macros::TryFromByte;

/// A byte opcode describes what the coming bytes on in a stack are
#[derive(TryFromByte, Debug)]
pub enum OpCode {
    /// Returns from a function
    ///
    /// Resets VM poiters to before the call, but keeping the return value on temp stack.
    /// Does discard all values on temp stack belonging to the call.
    Return,

    /// Pushes a constant to the temp stack
    ///
    /// The offset into the constant region is read from the next u8
    Constant,

    /// Special case of constant, for the NIL value
    Nil,

    /// Special case of constant, for the True bool
    True,

    /// Special case of constant, for the False bool
    False,

    /// Arithmetically negates the topmost value on the temp stack
    Negate,

    /// Logically negates the topmost value on the temp stack
    Not,

    /// Adds the top two value on the temp stack
    Add,

    /// Subtracts the topmost value on the temp stack from the second top-most value
    Subtract,

    /// Multiplies the top two value on the temp stack
    Multiply,

    /// Divides the second top-most value on the temp stack by the top value
    Divide,

    /// Modulo of the second top-most value on the temp stack by the top value
    Modulo,

    /// Raises the second top-most value on the temp stack to the power of the top value
    Power,

    /// Checks the two top-most values on the temp-stacks for equality
    Equality,

    /// Checks the two top-most values on the temp-stacks for non-equality
    NonEquality,

    /// Pushes whether the second top-most value is less than the top value
    LessThan,

    /// Pushes whether the second top-most value is less or equal to the top value
    LessEqual,

    /// Pushes whether the second top-most value is greater than the top value
    GreaterThan,

    /// Pushes whether the second top-most value is greater or equal to the top value
    GreaterEqual,

    /// Assigns to a global variable
    ///
    /// Reads the offset of the global variable from the next bytecode byte.
    /// The value is read from the temp stack, and it is kept on the stack.
    AssignGlobal,

    /// Reads a global variable onto the temp stack
    ///
    /// Reads the offset of the global variable from the next bytecode byte.
    ReadGlobal,

    /// Assigns to a local variable
    ///
    /// Reads the offset of the local variable from the rbp from the next bytecode byte.
    /// The value is read from the temp stack, and it is kept on the stack.
    AssignLocal,

    /// Reads a local variable onto the temp stack
    ///
    /// Reads the offset of the variable from the rbp from the next bytecode byte.
    ReadLocal,

    /// Assigns to a value closed over by a function
    ///
    /// The next byte specifies the index of the upvalue in the current closure.
    /// The value is read from the temp stack, and it is kept on the stack.
    AssignUpValue,

    /// Reads a value closed over by a function onto the temp stack
    ///
    /// The next byte specifies the index of the upvalue in the current closure.
    ReadUpValue,

    /// Assigns to a local value behind a pointer
    ///
    /// The next byte specifies the offset of the pointer from the rbp.
    AssignPointer,

    /// Reads the value behind a value pointer
    ///
    /// The next byte specifies the offset of the pointer from the rbp.
    ReadPointer,

    /// Jumps if the top value is false
    ///
    /// The jump offset for the pc is read as the next i16 in bytecode.
    /// The top value on the temp stack is consumed.
    JumpIfFalse,

    /// Jumps with target offset
    ///
    /// The jump offset for the pc is read as the next i16 in bytecode.
    Jump,

    /// Discards the top value on the temp stack
    Discard,

    /// Duplicates the top value on the temp stack
    Duplicate,

    /// Calls the top value on the temp stack
    ///
    /// Can be both a closure, and a builtin function.
    /// In case of a closure, it will create a new call-frame, and start exucuting
    /// the bytecode for the function. It will still have access to the same globals,
    /// but not local variables.
    Call,

    /// Intiates a closure from a function and upvalues
    ///
    /// The next byte specifies the constant index of the function to use init from.
    /// For every upvalue, there follows a byte for the index of the upvalue in the
    /// enclosing function (can only capture enclosing upvalues, which must be detected
    /// with semantic analysis).
    InitClosure,

    /// Drops the value at the next bytes offset
    ///
    /// Will be replaced by NIL, to not keep around pointers to old data.
    Drop,

    /// Pushes a new pointer to the temp-stack
    ///
    /// Used when declaring new pointers for future upvalues.
    EmptyPointer,

    /// Assigns a value at a certain (singular) index in a collection
    ///
    /// Collection[Index] = Value:
    ///     Index: The topmost stack value
    ///     Collection: The second topmost stack value
    ///     Value: The third topmost stack value
    AssignAtIndex,

    /// Reads a value at a certain (singular) index in a collection
    ///
    /// The topmost value is the index, and the second topmost is the collection
    ReadAtIndex,

    /// Reads and slice of a list
    ///
    /// The topmost 3 values is the slice, and the fourth is the list.
    /// If start or end is omitted, they default to 0 and the list length respectively.
    ReadAtSlice,

    /// Constructs a list from a pythonic slice
    ///
    /// The start, stop, step are on the stack in that order (NIL if omitted)
    ListFromSlice,

    /// Constructs a list from a computed set of values
    ///
    /// The following byte tells how many of the top values on the temp stack to use.
    ListFromValues,

    /// Converts the top value of the temp stack to something iterable
    TopToIter,

    /// Tries to get the next value from a collection. Jumps to label if it can't
    ///
    /// Next 2 bytes: offset to jump to
    /// Top of stack is the index to use, and below is the iterable value.
    /// In case of success: The index will be incremented and the indexed value pushed to the stack.
    /// In case of jump: Jump according to read bytes. The stack is left as it is
    NextOrJump,
}
