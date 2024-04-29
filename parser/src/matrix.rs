use std::{
    ops::{Add, Mul, Sub},
    process::Output,
};

use crate::prelude::*;

#[derive(PartialEq, Debug)]
pub struct Matrix<T> {
    // values[row][column]
    values: Vec<T>,
    row_count: usize,
    column_count: usize,
}

impl<T> Matrix<T> {
    pub(crate) fn new(values: Vec<T>, row_count: usize, column_count: usize) -> Self {
        if values.len() != row_count * column_count {
            panic!("values has incorrect size.")
        }
        Self {
            values,
            row_count,
            column_count,
        }
    }

    pub fn get(&self, row: usize, column: usize) -> &T {
        &self.values[row * self.column_count + column]
    }

    pub fn option_get(&self, row: usize, column: usize) -> Option<&T> {
        self.values.get(row * self.column_count + column)
    }

    pub fn set(&mut self, row: usize, column: usize, value: T) {
        self.values[row * self.column_count + column] = value;
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }
    pub fn column_count(&self) -> usize {
        self.column_count
    }
}

impl<T: Clone> Matrix<T> {
    pub fn new_default(row_count: usize, column_count: usize, default_value: T) -> Self {
        Self {
            values: vec![default_value; row_count * column_count],
            row_count,
            column_count,
        }
    }
}

impl<T: Clone> Clone for Matrix<T> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            row_count: self.row_count,
            column_count: self.column_count,
        }
    }
}

impl<In> Matrix<In> {
    fn map<F, Res>(&self, func: F) -> Result<Matrix<Res>, EvalError>
    where
        F: Fn(&In) -> Result<Res, EvalError>,
    {
        let val = self.values.iter().map(func).try_collect()?;
        Ok(Matrix::new(val, self.row_count, self.column_count))
    }

    fn pair_map<F, Res, Out>(&self, rhs: Matrix<Out>, func: F) -> Result<Matrix<Res>, EvalError>
    where
        F: Fn(In, Out) -> Result<Res, EvalError>,
        Out: Clone,
        In: Clone,
    {
        if self.row_count() != rhs.row_count() || self.column_count() != rhs.column_count() {
            return Err(EvalError::IncompatibleMatrixSizes);
        }
        let res = self
            .values
            .iter()
            .cloned()
            .zip(rhs.values.iter().cloned())
            .map(|a| func(a.0, a.1))
            .try_collect()?;
        Ok(Matrix::new(res, self.row_count, self.column_count))
    }
}

impl<Lhs, Res, Rhs> Add<Matrix<Rhs>> for Matrix<Lhs>
where
    Lhs: Clone + Add<Rhs, Output = Result<Res, EvalError>>,
    Rhs: Clone,
{
    type Output = Result<Matrix<Res>, EvalError>;

    fn add(self, rhs: Matrix<Rhs>) -> Self::Output {
        self.pair_map(rhs, |a, b| a + b)
    }
}

impl<Lhs: Clone + Sub<Rhs, Output = Result<Res, EvalError>>, Res, Rhs: Clone> Sub<Matrix<Rhs>>
    for Matrix<Lhs>
{
    type Output = Result<Matrix<Res>, EvalError>;

    fn sub(self, rhs: Matrix<Rhs>) -> Self::Output {
        self.pair_map(rhs, |a, b| a - b)
    }
}

impl<Rhs: Clone, Res, Lhs: Clone + Mul<Rhs, Output = Result<Res, EvalError>>> Mul<Rhs>
    for Matrix<Lhs>
{
    type Output = Result<Matrix<Res>, EvalError>;

    fn mul(self, rhs: Rhs) -> Self::Output {
        // Multiply matrix components by self
        self.map(|val| val.clone() * rhs.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;
    use crate::prelude::*;

    #[test]
    fn matrix_addition() {
        let a = Matrix::new_default(2, 3, Value::Scalar(1.0));
        let b = Matrix::new_default(2, 3, Value::Scalar(2.0));
        let c = Matrix::new_default(2, 3, Value::Scalar(3.0));

        assert_eq!((a + b).unwrap(), c);
    }
}
