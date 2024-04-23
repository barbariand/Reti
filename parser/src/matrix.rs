use std::ops::{Add, Mul, Sub};

use crate::{approximator::EvalError, value::Value};

#[derive(PartialEq, Debug)]
pub struct Matrix<T> {
    // values[row][column]
    values: Vec<Vec<T>>,
    row_count: usize,
    column_count: usize,
}

impl<T> Matrix<T> {
    pub fn get(&self, row: usize, column: usize) -> &T {
        &self.values[row][column]
    }

    pub fn option_get(&self, row: usize, column: usize) -> Option<&T> {
        self.values.get(row).and_then(|row| row.get(column))
    }

    pub fn set(&mut self, row: usize, column: usize, value: T) {
        (&mut self.values[row])[column] = value;
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }
    pub fn column_count(&self) -> usize {
        self.column_count
    }
}

impl<T: Clone> Matrix<T> {
    pub fn new(row_count: usize, column_count: usize, default_value: T) -> Self {
        Self {
            values: vec![vec![default_value; column_count]; row_count],
            row_count,
            column_count,
        }
    }
}

impl<T: Clone> Clone for Matrix<T> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
            row_count: self.row_count.clone(),
            column_count: self.column_count.clone(),
        }
    }
}

impl<T: Clone> Matrix<T> {
    fn map<F>(&self, mut func: F) -> Result<Matrix<T>, EvalError>
    where
        F: FnMut(&T) -> Result<T, EvalError>,
    {
        let mut result = self.clone();
        for row in 0..self.row_count {
            for col in 0..self.column_count {
                let current = self.get(row, col);
                let new = func(current)?;
                result.set(row, col, new);
            }
        }
        Ok(result)
    }

    fn pair_map<F>(&self, rhs: Matrix<T>, mut func: F) -> Result<Matrix<T>, EvalError>
    where
        F: FnMut(T, T) -> Result<T, EvalError>,
    {
        if self.row_count() != rhs.row_count() || self.column_count() != rhs.column_count() {
            return Err(EvalError::IncompatibleMatrixSizes);
        }
        let mut result = self.clone();
        for row in 0..self.row_count {
            for col in 0..self.column_count {
                let a = self.get(row, col).clone();
                let b = rhs.get(row, col).clone();
                let value = func(a, b)?;
                result.set(row, col, value);
            }
        }
        Ok(result)
    }
}

impl Add for Matrix<Value> {
    type Output = Result<Matrix<Value>, EvalError>;

    fn add(self, rhs: Self) -> Self::Output {
        self.pair_map(rhs, |a, b| a + b)
    }
}

impl Sub for Matrix<Value> {
    type Output = Result<Matrix<Value>, EvalError>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.pair_map(rhs, |a, b| a - b)
    }
}

impl Mul<Matrix<Value>> for f64 {
    type Output = Result<Matrix<Value>, EvalError>;

    fn mul(self, rhs: Matrix<Value>) -> Self::Output {
        // Multiply vector components by self
        rhs.map(|val| Value::Scalar(self) * val.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;
    use crate::value::Value;

    #[test]
    fn matrix_addition() {
        let a = Matrix::new(2, 3, Value::Scalar(1.0));
        let b = Matrix::new(2, 3, Value::Scalar(2.0));
        let c = Matrix::new(2, 3, Value::Scalar(3.0));

        assert_eq!((a + b).unwrap(), c);
    }
}
