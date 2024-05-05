//!
use std::ops::{Add, AddAssign, Mul, Sub};

use crate::prelude::*;

#[derive(PartialEq, Debug)]

pub struct Matrix<T> {
    // values[row][column]
    values: Vec<T>,
    row_count: usize,
    column_count: usize,
}

impl<T> Matrix<T> {
    /// Constructs a new `Matrix` instance.
    ///
    /// # Panics
    ///
    /// This constructor panics if the length of `values` is not equal to
    /// `row_count * column_count`. This ensures data consistency within the
    /// matrix and prevents potential errors later due to an incorrect
    /// underlying representation.
    pub fn new(values: Vec<T>, row_count: usize, column_count: usize) -> Self {
        if values.len() != row_count * column_count {
            panic!("values has incorrect size.") //should it be a panic or an
                                                 // err? i mean its probably a
                                                 // parse error here but i think
                                                 // we want that as a result
        }
        Self {
            values,
            row_count,
            column_count,
        }
    }

    /// Accesses the element at the specified `row` and `column`.
    ///
    /// # Returns
    ///
    /// A reference to the element at the given index.
    pub fn get(&self, row: usize, column: usize) -> &T {
        &self.values[row * self.column_count + column]
    }

    /// Accesses the element at the specified `row` and `column` with an
    /// optional return type.
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if the element exists.
    /// - `None` if the index is out of bounds.
    pub fn option_get(&self, row: usize, column: usize) -> Option<&T> {
        self.values.get(row * self.column_count + column)
    }

    /// Sets the element at the specified `row` and `column`.
    pub fn set(&mut self, row: usize, column: usize, value: T) {
        self.values[row * self.column_count + column] = value;
    }

    /// Returns the total number of rows in the matrix.
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Returns the total number of columns in the matrix.
    pub fn column_count(&self) -> usize {
        self.column_count
    }
}

impl<T: Clone> Matrix<T> {
    /// Constructs a new `Matrix` instance filled with a default value.
    pub fn new_default(row_count: usize, column_count: usize, default_value: T) -> Self {
        Self {
            values: vec![default_value; row_count * column_count],
            row_count,
            column_count,
        }
    }
    /// Calculates the transpose of the matrix.
    ///
    /// # Returns
    ///
    /// A new `Matrix` that represents the transpose of the original matrix.
    pub fn transpose(&self) -> Matrix<T> {
        let mut transposed_values = Vec::with_capacity(self.row_count * self.column_count);

        for column in 0..self.column_count {
            for row in 0..self.row_count {
                transposed_values.push(self.values[row * self.column_count + column].clone());
            }
        }

        Matrix {
            values: transposed_values,
            row_count: self.column_count, // swap row and column counts
            column_count: self.row_count,
        }
    }
}
impl<Lhs> Matrix<Lhs> {
    /// Calculates the dot product of two matrices (treated as vectors).
    ///
    /// # Errors
    ///
    /// Returns an `Err` if:
    ///    - Both matrices are not vectors (i.e., more than one row or column).
    ///    - The dimensions of the matrices do not match for dot product
    ///      calculation.
    pub fn dot_product<Rhs, Res>(&self, other: &Matrix<Rhs>) -> Result<Res, &'static str>
    where
        Lhs: Mul<Rhs, Output = Res> + Clone,
        Res: AddAssign + Default,
        Rhs: Clone,
    {
        if self.row_count != other.row_count || self.column_count != 1 || other.column_count != 1 {
            return Err("Both matrices must be vectors of the same dimension.");
        }

        let mut result = Default::default();
        for i in 0..self.row_count {
            result += self.values[i].clone() * other.values[i].clone();
        }

        Ok(result)
    }
    /// Calculates the cross product of two 3D vectors represented as matrices.
    ///
    /// # Errors
    ///
    /// Returns an `Err` of type `IncompatibleMatrixSizes` if:
    ///    - Either matrix does not have exactly 3 rows.
    ///    - Either matrix does not have exactly 1 column.
    pub fn cross_product<Rhs, Res>(&self, other: &Matrix<Rhs>) -> Result<Matrix<Res>, EvalError>
    where
        Lhs: Mul<Rhs, Output = Res> + Clone,
        Res: AddAssign + Sub<Output = Res>,
        Rhs: Clone,
    {
        if self.row_count != 3 {
            return Err(IncompatibleMatrixSizes::Column {
                expected: 3,
                found: self.column_count,
            }
            .into());
        }
        if self.column_count != 1 {
            return Err(IncompatibleMatrixSizes::Column {
                expected: 1,
                found: self.column_count,
            }
            .into());
        }
        if other.row_count != 3 {
            return Err(IncompatibleMatrixSizes::Column {
                expected: 3,
                found: other.column_count,
            }
            .into());
        }
        if other.column_count != 1 {
            return Err(IncompatibleMatrixSizes::Column {
                expected: 1,
                found: other.column_count,
            }
            .into());
        }

        Ok(Matrix {
            values: vec![
                self.values[1].clone() * other.values[2].clone()
                    - self.values[2].clone() * other.values[1].clone(), // Cx
                self.values[2].clone() * other.values[0].clone()
                    - self.values[0].clone() * other.values[2].clone(), // Cy
                self.values[0].clone() * other.values[1].clone()
                    - self.values[1].clone() * other.values[0].clone(), // Cz
            ],
            row_count: 3,
            column_count: 1,
        })
    }
    /// Calculates the determinant of a square matrix.
    ///
    /// # Errors
    ///
    /// Returns an `Err` of type `EvalError` if the matrix is not square
    pub fn determinant<Res>(&self) -> Result<Res, EvalError>
    where
        Lhs: Clone + Mul<Res, Output = Result<Res, EvalError>> + Into<Res>,
        Res: Default + AddAssign,
    {
        if self.row_count != self.column_count {
            return Err(IncompatibleMatrixSizes::Column {
                expected: self.row_count,
                found: self.column_count,
            }
            .into());
        }
        if self.row_count == 1 {
            return Ok(self.values[0].clone().into());
        }

        let mut det = Res::default();
        for col in 0..self.column_count {
            let submatrix = self.submatrix(0, col);
            let term = (self.get(0, col).clone() * (submatrix.determinant()?))?;
            det += term;
        }

        Ok(det)
    }

    /// Helper function to create a submatrix by excluding a specified row and
    /// column.
    fn submatrix(&self, skip_row: usize, skip_col: usize) -> Matrix<Lhs>
    where
        Lhs: Clone,
    {
        let mut sub_values = Vec::new();
        for row in 0..self.row_count {
            if row == skip_row {
                continue;
            }
            for col in 0..self.column_count {
                if col == skip_col {
                    continue;
                }
                sub_values.push(self.get(row, col).clone());
            }
        }
        Matrix {
            values: sub_values,
            row_count: self.row_count - 1,
            column_count: self.column_count - 1,
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
    /// Maps a function over all elements of the matrix.
    ///
    /// # Errors
    ///
    /// Returns an `Err` of type `EvalError` if the provided function `func`
    /// returns an error for any of the matrix elements.
    fn map<F, Res>(&self, func: F) -> Result<Matrix<Res>, EvalError>
    where
        F: Fn(&T) -> Result<Res, EvalError>,
    {
        let val: Result<_, _> = self.values.iter().map(func).collect();
        Ok(Matrix::new(val?, self.row_count, self.column_count))
    }
    /// Performs element-wise operations with another matrix using a provided
    /// function.
    ///
    /// # Errors
    ///
    /// Returns an `Err` of type `IncompatibleMatrixSizes` if the dimensions of
    /// the two matrices do not match. Returns an `Err` of type `EvalError`
    /// if the provided function `func` returns an error for any of the
    /// element pairs.
    fn pair_map<F, Res, Out>(&self, rhs: Matrix<Out>, func: F) -> Result<Matrix<Res>, EvalError>
    where
        F: Fn(T, Out) -> Result<Res, EvalError>,
        Out: Clone,
        T: Clone,
    {
        if self.row_count() != rhs.row_count() {
            return Err(IncompatibleMatrixSizes::Row {
                expected: self.row_count,
                found: rhs.row_count,
            }
            .into());
        }
        if self.column_count() != rhs.column_count() {
            return Err(IncompatibleMatrixSizes::Column {
                expected: self.column_count,
                found: rhs.column_count,
            }
            .into());
        }
        let res: Result<_, _> = self
            .values
            .iter()
            .cloned()
            .zip(rhs.values.iter().cloned())
            .map(|a| func(a.0, a.1))
            .collect();
        Ok(Matrix::new(res?, self.row_count, self.column_count))
    }
}
impl<Lhs, Rhs, Res> Mul<Matrix<Rhs>> for Matrix<Lhs>
where
    Lhs: Mul<Rhs, Output = Res> + Clone,
    Rhs: Clone,
    Res: Default + Clone + AddAssign,
{
    type Output = Result<Matrix<Res>, EvalError>;
    // Matrix multiplication
    fn mul(self, other: Matrix<Rhs>) -> Result<Matrix<Res>, EvalError> {
        if self.column_count != other.row_count {
            return Err(IncompatibleMatrixSizes::Row {
                expected: self.column_count,
                found: other.row_count,
            }
            .into());
        }

        let mut result_values = vec![Res::default(); self.row_count * other.column_count];
        for i in 0..self.row_count {
            for j in 0..other.column_count {
                let mut sum = Res::default();
                for k in 0..self.column_count {
                    sum += self.values[i * self.column_count + k].clone()
                        * other.values[k * other.column_count + j].clone();
                }
                result_values[i * other.column_count + j] = sum;
            }
        }

        Ok(Matrix {
            values: result_values,
            row_count: self.row_count,
            column_count: other.column_count,
        })
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

impl<Lhs, Res, Rhs> Sub<Matrix<Rhs>> for Matrix<Lhs>
where
    Lhs: Clone + Sub<Rhs, Output = Result<Res, EvalError>>,
    Rhs: Clone,
{
    type Output = Result<Matrix<Res>, EvalError>;

    fn sub(self, rhs: Matrix<Rhs>) -> Self::Output {
        self.pair_map(rhs, |a, b| a - b)
    }
}

impl<Lhs: Clone + Mul<Value, Output = Result<Value, EvalError>>> Mul<Value> for Matrix<Lhs> {
    type Output = Result<Matrix<Value>, EvalError>;

    fn mul(self, rhs: Value) -> Self::Output {
        // Multiply matrix components by self
        self.map(|val| val.clone() * rhs.clone())
    }
}
impl<Lhs: Clone + Mul<f64, Output = Result<Value, EvalError>>> Mul<f64> for Matrix<Lhs> {
    type Output = Result<Matrix<Value>, EvalError>;

    fn mul(self, rhs: f64) -> Self::Output {
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
