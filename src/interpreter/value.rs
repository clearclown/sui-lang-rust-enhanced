//! Value types for the Sui language

use std::fmt;

/// Sui runtime value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Array value
    Array(Vec<Value>),
    /// Null/None value
    Null,
}

impl Value {
    /// Convert value to boolean (0 or empty = false, otherwise true)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Integer(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            Value::Null => false,
        }
    }

    /// Convert to integer
    pub fn to_int(&self) -> i64 {
        match self {
            Value::Integer(n) => *n,
            Value::Float(f) => *f as i64,
            Value::String(s) => s.parse().unwrap_or(0),
            Value::Array(arr) => arr.len() as i64,
            Value::Null => 0,
        }
    }

    /// Convert to float
    pub fn to_float(&self) -> f64 {
        match self {
            Value::Integer(n) => *n as f64,
            Value::Float(f) => *f,
            Value::String(s) => s.parse().unwrap_or(0.0),
            Value::Array(arr) => arr.len() as f64,
            Value::Null => 0.0,
        }
    }

    /// Check if this value is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Float(_))
    }

    /// Add two values
    pub fn add(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 + b),
            (Value::Float(a), Value::Integer(b)) => Value::Float(a + *b as f64),
            (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
            _ => Value::Float(self.to_float() + other.to_float()),
        }
    }

    /// Subtract two values
    pub fn sub(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 - b),
            (Value::Float(a), Value::Integer(b)) => Value::Float(a - *b as f64),
            _ => Value::Float(self.to_float() - other.to_float()),
        }
    }

    /// Multiply two values
    pub fn mul(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => Value::Integer(a * b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 * b),
            (Value::Float(a), Value::Integer(b)) => Value::Float(a * *b as f64),
            _ => Value::Float(self.to_float() * other.to_float()),
        }
    }

    /// Divide two values
    pub fn div(&self, other: &Value) -> Value {
        let divisor = other.to_float();
        if divisor == 0.0 {
            return Value::Float(f64::NAN);
        }
        Value::Float(self.to_float() / divisor)
    }

    /// Modulo two values
    pub fn modulo(&self, other: &Value) -> Value {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) if *b != 0 => Value::Integer(a % b),
            _ => {
                let divisor = other.to_float();
                if divisor == 0.0 {
                    Value::Float(f64::NAN)
                } else {
                    Value::Float(self.to_float() % divisor)
                }
            }
        }
    }

    /// Less than comparison
    pub fn lt(&self, other: &Value) -> Value {
        let result = match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a < b,
            (Value::String(a), Value::String(b)) => a < b,
            _ => self.to_float() < other.to_float(),
        };
        Value::Integer(if result { 1 } else { 0 })
    }

    /// Greater than comparison
    pub fn gt(&self, other: &Value) -> Value {
        let result = match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a > b,
            (Value::String(a), Value::String(b)) => a > b,
            _ => self.to_float() > other.to_float(),
        };
        Value::Integer(if result { 1 } else { 0 })
    }

    /// Equality comparison
    pub fn eq_val(&self, other: &Value) -> Value {
        let result = match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => self.to_float() == other.to_float(),
        };
        Value::Integer(if result { 1 } else { 0 })
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Integer(0)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => {
                // Format like Python - remove trailing zeros
                if n.fract() == 0.0 {
                    write!(f, "{}.0", n.trunc())
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Null => write!(f, "null"),
        }
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Integer(n)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Float(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<Vec<Value>> for Value {
    fn from(arr: Vec<Value>) -> Self {
        Value::Array(arr)
    }
}
