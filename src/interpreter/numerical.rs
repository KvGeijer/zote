use core::cmp::max;

use super::runtime_error::{RunError, RunRes};

#[derive(Debug, Clone, Copy)]
pub enum Numerical {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Numerical {
    pub fn truthy(&self) -> bool {
        match self {
            Numerical::Bool(bool) => *bool,
            Numerical::Int(int) => *int != 0,
            Numerical::Float(float) => *float != 0.0,
        }
    }

    pub fn stringify(&self) -> String {
        // OPT Could we just return &str here?
        match self {
            Numerical::Bool(bool) => format!("{bool}"),
            Numerical::Int(int) => format!("{int}"),
            Numerical::Float(float) => format!("{float}"),
        }
    }

    pub fn type_of(&self) -> &'static str {
        match self {
            Numerical::Bool(_) => "Bool",
            Numerical::Int(_) => "Int",
            Numerical::Float(_) => "Float",
        }
    }

    fn promotion_level(&self) -> u8 {
        match self {
            Numerical::Bool(_) => 0,
            Numerical::Int(_) => 1,
            Numerical::Float(_) => 2,
        }
    }

    pub fn to_float(self) -> Numerical {
        match self {
            Numerical::Bool(true) => 1.0,
            Numerical::Bool(false) => 0.0,
            Numerical::Int(int) => int as f64,
            Numerical::Float(float) => float,
        }
        .into()
    }

    pub fn to_int(self) -> Numerical {
        Numerical::Int(self.to_rint())
    }

    pub fn to_rint(self) -> i64 {
        match self {
            Numerical::Bool(true) => 1,
            Numerical::Bool(false) => 0,
            Numerical::Int(int) => int,
            Numerical::Float(float) => float as i64,
        }
    }

    pub fn add(self, other: Numerical) -> Numerical {
        match math_promote(&self, &other) {
            (Numerical::Int(x), Numerical::Int(y)) => Numerical::Int(x + y),
            (Numerical::Float(x), Numerical::Float(y)) => Numerical::Float(x + y),
            _ => panic!("Internal error with math_promote"),
        }
    }

    pub fn sub(self, other: Numerical) -> Numerical {
        match math_promote(&self, &other) {
            (Numerical::Int(x), Numerical::Int(y)) => Numerical::Int(x - y),
            (Numerical::Float(x), Numerical::Float(y)) => Numerical::Float(x - y),
            _ => panic!("Internal error with math_promote"),
        }
    }

    pub fn mult(self, other: Numerical) -> Numerical {
        match math_promote(&self, &other) {
            (Numerical::Int(x), Numerical::Int(y)) => Numerical::Int(x * y),
            (Numerical::Float(x), Numerical::Float(y)) => Numerical::Float(x * y),
            _ => panic!("Internal error with math_promote"),
        }
    }

    pub fn div(self, other: Numerical) -> RunRes<Numerical> {
        match math_promote(&self, &other) {
            (_, Numerical::Int(0)) => RunError::error("Cannot divide int by 0".to_string()),
            (Numerical::Int(x), Numerical::Int(y)) => Ok(Numerical::Int(x / y)),
            (Numerical::Float(x), Numerical::Float(y)) => Ok(Numerical::Float(x / y)),
            _ => panic!("Internal error with math_promote"),
        }
    }

    pub fn modulo(self, other: Numerical) -> Numerical {
        match math_promote(&self, &other) {
            (Numerical::Int(x), Numerical::Int(y)) => Numerical::Int(x.rem_euclid(y)),
            (Numerical::Float(x), Numerical::Float(y)) => Numerical::Float(x.rem_euclid(y)),
            _ => panic!("Internal error with math_promote"),
        }
    }

    pub fn pow(self, other: Numerical) -> Numerical {
        match math_promote(&self, &other) {
            (Numerical::Float(x), Numerical::Float(y)) => Numerical::Float(x.powf(y)),
            (Numerical::Int(x), Numerical::Int(y)) if y >= 0 => {
                let safe_x: u64 = x.unsigned_abs(); // TODO Handle overflows as zote errors
                let pow = safe_x.pow(y.unsigned_abs() as u32) as i64;
                if x >= 0 || y & 1 == 0 {
                    Numerical::Int(pow)
                } else {
                    Numerical::Int(-pow)
                }
            }
            (Numerical::Int(x), Numerical::Int(y)) => Numerical::Float((x as f64).powf(y as f64)),
            _ => panic!("Internal error with math_promote"),
        }
    }

    pub fn un_sub(self) -> RunRes<Numerical> {
        match self {
            Numerical::Int(int) => Ok(Numerical::Int(-int)),
            Numerical::Float(float) => Ok(Numerical::Float(-float)),
            Numerical::Bool(_) => RunError::error("Cannot negate a bool".to_string()),
        }
    }

    pub fn abs(self) -> Numerical {
        match self {
            Numerical::Int(int) => Numerical::Int(int.abs()),
            Numerical::Float(float) => Numerical::Float(float.abs()),
            bool => bool,
        }
    }
}

impl PartialEq for Numerical {
    fn eq(&self, other: &Numerical) -> bool {
        match promote(self, other) {
            (Numerical::Int(x), Numerical::Int(y)) => x == y,
            (Numerical::Float(x), Numerical::Float(y)) => x == y,
            (Numerical::Bool(x), Numerical::Bool(y)) => x == y,
            _ => panic!("Internal error with promote!"),
        }
    }
}

impl PartialOrd for Numerical {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match promote(self, other) {
            (Numerical::Int(x), Numerical::Int(y)) => x.partial_cmp(&y),
            (Numerical::Float(x), Numerical::Float(y)) => x.partial_cmp(&y),
            (Numerical::Bool(x), Numerical::Bool(y)) => x.partial_cmp(&y),
            _ => panic!("Internal error with promote"),
        }
    }
}

impl From<i64> for Numerical {
    fn from(item: i64) -> Self {
        Numerical::Int(item)
    }
}

impl From<f64> for Numerical {
    fn from(item: f64) -> Self {
        Numerical::Float(item)
    }
}

impl From<bool> for Numerical {
    fn from(item: bool) -> Self {
        Numerical::Bool(item)
    }
}

fn promote(x: &Numerical, y: &Numerical) -> (Numerical, Numerical) {
    match max(x.promotion_level(), y.promotion_level()) {
        0 => (*x, *y),
        1 => (x.to_int(), y.to_int()),
        2 => (x.to_float(), y.to_float()),
        _ => panic!("internal error in promotion level"),
    }
}

fn math_promote(x: &Numerical, y: &Numerical) -> (Numerical, Numerical) {
    match max(x.promotion_level(), y.promotion_level()) {
        0 => (x.to_int(), y.to_int()),
        1 => (x.to_int(), y.to_int()),
        2 => (x.to_float(), y.to_float()),
        _ => panic!("internal error in promotion level"),
    }
}
