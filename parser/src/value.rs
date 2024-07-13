//!Managing the different values that can exist
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::{ast::MulType, error::EvalError, matrix::Matrix};

///The different types of values that can exist
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    ///A normal number
    Scalar(f64),
    ///A matrix of Values
    Matrix(Matrix<Value>),
}

impl Value {
    ///returns as a scalar or fails
    #[inline(always)]
    pub fn scalar(&self) -> Result<f64, EvalError> {
        match self {
            Value::Scalar(val) => Ok(*val),
            Value::Matrix(m) => {
                // Treat a 1x1 matrix as a scalar.
                if m.row_count() == 1 && m.column_count() == 1 {
                    return m.get(0, 0).scalar();
                }
                Err(EvalError::ExpectedScalar)
            }
        }
    }
    /// mapping it for a function where it is a scalar and if not it returns an
    /// error
    #[inline(always)]
    pub fn map_expecting_scalar(
        &self,
        func: impl Fn(f64) -> f64,
    ) -> Result<Value, EvalError> {
        match self {
            Value::Scalar(v) => Ok(Value::Scalar(func(*v))),
            Value::Matrix(_) => Err(EvalError::ExpectedScalar),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Scalar(val) => write!(f, "{}", val),
            Value::Matrix(m) => write!(f, "{:?}", m), // TODO
        }
    }
}
///Creates an incompatible type with the expected message
fn type_err<T>(text: &'static str) -> Result<T, EvalError> {
    Err(EvalError::IncompatibleTypes {
        message: text.to_string(),
    })
}

impl Add for Value {
    type Output = Result<Value, EvalError>;

    fn add(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Scalar(a), Value::Scalar(b)) => Value::Scalar(a + b),
            (Value::Matrix(a), Value::Matrix(b)) => Value::Matrix((a + b)?),
            (Value::Scalar(_), Value::Matrix(_)) => {
                return type_err("Cannot add a scalar and a matrix.")
            }
            (Value::Matrix(_), Value::Scalar(_)) => {
                return type_err("Cannot add a matrix and a scalar.")
            }
        })
    }
}

impl Sub for Value {
    type Output = Result<Value, EvalError>;

    fn sub(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Scalar(a), Value::Scalar(b)) => Value::Scalar(a - b),
            (Value::Matrix(a), Value::Matrix(b)) => Value::Matrix((a - b)?),
            (Value::Scalar(_), Value::Matrix(_)) => {
                return type_err("Cannot subtract a scalar and a matrix.")
            }
            (Value::Matrix(_), Value::Scalar(_)) => {
                return type_err("Cannot subtract a matrix and a scalar.")
            }
        })
    }
}

impl Value {
    /// multiplication with the different types of multiplication
    pub fn mul(
        &self,
        mul_type: &MulType,
        rhs: &Self,
    ) -> Result<Value, EvalError> {
        Ok(match (self, rhs) {
            (Value::Scalar(a), Value::Scalar(b)) => Value::Scalar(a * b),
            (Value::Matrix(a), Value::Matrix(b)) => match mul_type {
                MulType::Implicit => Value::Matrix((a.matrix_mul(b))?),
                MulType::Cdot => a.dot_product(b)?,
                MulType::Times => Value::Matrix(a.cross_product(b)?),
                _ => {
                    return Err(EvalError::AmbiguousMulType {
                        r#type: mul_type.clone(),
                    })
                }
            },
            (Value::Scalar(scalar), Value::Matrix(matrix)) => {
                Value::Matrix((matrix.mul(*scalar))?)
            }
            (Value::Matrix(matrix), Value::Scalar(scalar)) => {
                Value::Matrix((matrix * *scalar)?)
            }
        })
    }
}

impl Div for Value {
    type Output = Result<Value, EvalError>;

    fn div(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Scalar(a), Value::Scalar(b)) => Value::Scalar(a / b),
            (_, _) => {
                return type_err("Cannot perform division with matrices.")
            }
        })
    }
}

impl Mul<f64> for &Value {
    type Output = Result<Value, EvalError>;

    fn mul(self, rhs: f64) -> Self::Output {
        Value::Scalar(rhs).mul(&MulType::Implicit, self)
    }
}
