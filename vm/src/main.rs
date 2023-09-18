// use parser::CodeRange;

// use vm::compiler::OpCode;
use vm::compiler::Compiler;
use vm::disassembler;
use vm::interpreter::interpret;
// use vm::value::Value;

// fn push_opcode(chunk: &mut Chunk, opcode: OpCode) {
//     chunk.push_opcode(opcode, CodeRange::from_ints(0, 0, 0, 0, 0, 0));
// }

// fn push_const(chunk: &mut Chunk, constant: Value) {
//     push_opcode(chunk, OpCode::Constant);
//     chunk.push_constant(constant);
// }

pub fn main() {
    test_num_ops_str();

    // test_num_ops();
}

fn test_num_ops_str() {
    let mut compiler = Compiler::new();
    // let string = "5+2^2*3 - 4 % 5";
    let string = "x := -1; return x; return ((true + 3) < 5) - false - 1 == 0";
    let stmts = parser::parse(string).unwrap();
    let chunk = compiler.compile(&stmts).unwrap();
    disassembler::disassemble_chunk(&chunk, "main chunk", &mut std::io::stdout()).unwrap();
    println!("\n== Running chunk ==");
    interpret(&chunk, true).unwrap();
}

// fn test_num_ops() {
//     println!("=== TESTING NUMERICAL OPS ===");
//     let mut chunk = Chunk::new();
//     push_const(&mut chunk, Value::Int(13));
//     push_const(&mut chunk, Value::Int(37));
//     push_opcode(&mut chunk, OpCode::Subtract);
//     push_opcode(&mut chunk, OpCode::Negate);
//     push_const(&mut chunk, Value::Int(42));
//     push_const(&mut chunk, Value::Int(37));
//     push_const(&mut chunk, Value::Int(3));
//     push_opcode(&mut chunk, OpCode::Add);
//     push_opcode(&mut chunk, OpCode::Subtract);
//     push_opcode(&mut chunk, OpCode::Multiply);
//     push_const(&mut chunk, Value::Int(13));
//     push_const(&mut chunk, Value::Int(-7));
//     push_opcode(&mut chunk, OpCode::Divide);
//     disassembler::disassemble_chunk(&chunk, "main chunk", &mut std::io::stdout()).unwrap();
//     println!("\n== Running chunk ==");
//     interpret(&chunk, true).unwrap();
// }
