use std::ops::{Add, Mul, Sub};

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
            panic!(
                "values has incorrect size. Expected {} ({}*{}), found {}",
                row_count * column_count,
                row_count,
                column_count,
                values.len()
            )
        }
        Self {
            values,
            row_count,
            column_count,
        }
    }

    pub fn index(&self, row: usize, column: usize) -> usize {
        if row >= self.row_count {
            panic!("Row out out bounds. {}/{}", row, self.row_count);
        }
        if column >= self.column_count {
            panic!("Column out out bounds. {}/{}", column, self.column_count);
        }
        return row * self.column_count + column;
    }

    pub fn get(&self, row: usize, column: usize) -> &T {
        &self.values[self.index(row, column)]
    }

    pub fn option_get(&self, row: usize, column: usize) -> Option<&T> {
        self.values.get(self.index(row, column))
    }

    pub fn set(&mut self, row: usize, column: usize, value: T) {
        let index = self.index(row, column);
        self.values[index] = value;
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

impl Matrix<MathExpr> {
    pub fn zero(row_count: usize, column_count: usize) -> Self {
        let mut values = Vec::with_capacity(row_count * column_count);
        for _ in 0..(row_count * column_count) {
            values.push(0f64.into());
        }
        Self {
            values,
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

impl<T> Matrix<T> {
    pub fn map<F, R>(&self, func: F) -> Result<Matrix<R>, EvalError>
    where
        F: FnMut(&T) -> Result<R, EvalError>,
    {
        let mapped_values: Result<Vec<_>, _> = self.values.iter().map(func).collect();
        Ok(Matrix::new(
            mapped_values?,
            self.row_count,
            self.column_count,
        ))
    }

    fn pair_map<F>(&self, rhs: Matrix<T>, mut func: F) -> Result<Matrix<T>, EvalError>
    where
        F: FnMut(&T, &T) -> Result<T, EvalError>,
    {
        if self.row_count() != rhs.row_count() || self.column_count() != rhs.column_count() {
            return Err(EvalError::IncompatibleMatrixSizes);
        }
        let mut result_values = Vec::with_capacity(self.values.len());
        for (a, b) in self.values.iter().zip(rhs.values.iter()) {
            let value = func(a, b)?;
            result_values.push(value);
        }
        Ok(Matrix::new(
            result_values,
            self.row_count,
            self.column_count,
        ))
    }
}

impl Add for Matrix<Value> {
    type Output = Result<Matrix<Value>, EvalError>;

    fn add(self, rhs: Self) -> Self::Output {
        self.pair_map(rhs, |a, b| a.clone() + b.clone())
    }
}

impl Sub for Matrix<Value> {
    type Output = Result<Matrix<Value>, EvalError>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.pair_map(rhs, |a, b| a.clone() - b.clone())
    }
}

impl Mul<Matrix<Value>> for f64 {
    type Output = Result<Matrix<Value>, EvalError>;

    fn mul(self, rhs: Matrix<Value>) -> Self::Output {
        // Multiply matrix components by self
        rhs.map(|val| Value::Scalar(self) * val.clone())
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
