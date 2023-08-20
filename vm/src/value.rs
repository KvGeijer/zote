// OPT: Pack as bytesting instead? Very inefficiently stored now in 128 bits
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Float(f64),
    Int(i64),
}
