use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::{approximator::EvalError, matrix::Matrix};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Scalar(f64),
    Matrix(Matrix<Value>),
}

impl Value {
    pub fn scalar(&self) -> Result<f64, EvalError> {
        match self {
            Value::Scalar(val) => Ok(*val),
            Value::Matrix(_) => Err(EvalError::ExpectedScalar),
        }
    }
    pub fn map_scalar(&self, func: impl Fn(&f64) -> f64) -> Result<Value, EvalError> {
        match self {
            Value::Scalar(v) => Ok(Value::Scalar(func(v))),
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

fn type_err<T>(text: &'static str) -> Result<T, EvalError> {
    Err(EvalError::IncompatibleTypes(text))
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

impl Mul for Value {
    type Output = Result<Value, EvalError>;

    fn mul(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Scalar(a), Value::Scalar(b)) => Value::Scalar(a * b),
            (Value::Matrix(_a), Value::Matrix(_b)) => todo!("Matrix multiplication"),
            (Value::Scalar(scalar), Value::Matrix(matrix)) => Value::Matrix((scalar * matrix)?),
            (Value::Matrix(matrix), Value::Scalar(scalar)) => Value::Matrix((scalar * matrix)?),
        })
    }
}

impl Div for Value {
    type Output = Result<Value, EvalError>;

    fn div(self, rhs: Self) -> Self::Output {
        Ok(match (self, rhs) {
            (Value::Scalar(a), Value::Scalar(b)) => Value::Scalar(a / b),
            (_, _) => return type_err("Cannot perform division with matricies."),
        })
    }
}
