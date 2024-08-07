//! # Matrix
//!
//! implementing all the matrix multiplication scalar or otherwise
use std::ops::{Add, AddAssign, Mul, Sub};

use crate::prelude::*;

///The matrix struct representing a Matrix with one or more rows and columns
#[derive(PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Matrix<T> {
    /// all the values stored in a Vec stored column per column so first in:
    /// values\[row*column_count+column]
    values: Vec<T>,
    ///The amount of rows in the Vec
    row_count: usize,
    ///The amount of columns in the Vec
    column_count: usize,
}

/// A vector.
///
/// This struct is a wrapper around a matrix that only has one row or column.
///
/// # Lifetime
/// The vector wraps the matrix, and the lifetime of the vector must therefore
/// be shorter than the lifetime of the matrix.
pub enum Vector<'a, T> {
    /// A row vector is a matrix with only one row.
    Row(&'a Matrix<T>),
    /// A column vector is a matrix with only one column.
    Column(&'a Matrix<T>),
}

impl<'a, T> Vector<'a, T> {
    /// Create a new vector that wraps a matrix.
    ///
    /// # Errors
    /// Will return an `IncompatibleMatrixSizes::Vector` `EvalError` if the
    /// matrix is not a row or column vector.
    pub const fn new(matrix: &'a Matrix<T>) -> Result<Self, EvalError> {
        if matrix.is_row_vector() {
            return Ok(Vector::Row(matrix));
        } else if matrix.is_column_vector() {
            return Ok(Vector::Column(matrix));
        }
        Err(EvalError::IncompatibleMatrixSizes {
            source: IncompatibleMatrixSizes::Vector {
                rows: matrix.row_count(),
                columns: matrix.column_count(),
            },
        })
    }

    /// Get the amount of elements this vector has.
    pub const fn get_size(&self) -> usize {
        match self {
            Vector::Row(matrix) => matrix.column_count,
            Vector::Column(matrix) => matrix.row_count,
        }
    }

    /// Get the elements at an index in this vector.
    pub fn get(&self, index: usize) -> &T {
        match self {
            Vector::Row(matrix) => matrix.get(0, index),
            Vector::Column(matrix) => matrix.get(index, 0),
        }
    }
}
impl<T> Matrix<T> {
    /// Constructs a new `Matrix` instance from a vector containing the values
    ///
    /// In the Matrix internal representation the data as a Vec<T> witch is why
    /// it is required to provide a Vec<T> instead of Vec<Vec<T>>
    ///
    /// Note that this means that the elements need to be column per column so
    /// that indexing an element in row 1 and column 2 with size
    /// 3(row)x4(column) its 1 * 4 + 2=6
    ///
    /// # Panics
    ///
    /// This constructor panics if the length of `values` is not equal to
    /// `row_count * column_count`. This ensures data consistency within the
    /// matrix and prevents potential errors later due to an incorrect
    /// underlying representation.
    ///
    /// Will also panic if the Matrix has a row or column count of zero.
    pub fn new(values: Vec<T>, row_count: usize, column_count: usize) -> Self {
        if values.len() != row_count * column_count {
            panic!(
                "values has incorrect size. Expected {} ({}*{}), found {}",
                row_count * column_count,
                row_count,
                column_count,
                values.len()
            );
            //should it be a panic or an err? i mean its probably a parse error
            // here but i think we want that as a result No I think
            // it should be a panic. If this is called with an incorrect vec
            // that is a bug that we need to fix.
        }
        if row_count == 0 || column_count == 0 {
            panic!("Empty matrix.");
        }
        Self {
            values,
            row_count,
            column_count,
        }
    }
    ///Calculate the index for the Matrix given the row number and column
    /// number
    ///
    /// # Panics
    /// if the row or column is bigger than the row_count or column_count given
    /// when instantiated it will be considered out of bounds witch is most
    /// likely to be a bug
    fn index(&self, row: usize, column: usize) -> usize {
        if row >= self.row_count {
            panic!("Row out out bounds. {}/{}", row, self.row_count);
        }
        if column >= self.column_count {
            panic!("Column out out bounds. {}/{}", column, self.column_count);
        }
        row * self.column_count + column
    }

    /// Accesses the element at the specified `row` and `column`.
    ///
    /// # Returns
    ///
    /// A reference to the element at the given index.
    pub fn get(&self, row: usize, column: usize) -> &T {
        &self.values[self.index(row, column)]
    }

    /// Accesses the element at the specified `row` and `column` with an
    /// optional return type.
    ///
    /// # Returns
    ///
    /// - `Some(&T)` if the element exists.
    /// - `None` if the index is out of bounds.
    pub fn option_get(&self, row: usize, column: usize) -> Option<&T> {
        self.values.get(self.index(row, column))
    }

    /// Sets the element at the specified `row` and `column`.
    pub fn set(&mut self, row: usize, column: usize, value: T) {
        let index = self.index(row, column);
        self.values[index] = value;
    }

    /// Returns the total number of rows in the matrix.
    pub const fn row_count(&self) -> usize {
        self.row_count
    }

    /// Returns the total number of columns in the matrix.
    pub const fn column_count(&self) -> usize {
        self.column_count
    }

    /// Returns whether this matrix is a vector. (Could be a row vector
    /// or a column vector).
    pub const fn is_vector(&self) -> bool {
        self.is_row_vector() || self.is_column_vector()
    }

    /// Returns whether this matrix is a row vector.
    pub const fn is_row_vector(&self) -> bool {
        self.row_count == 1
    }

    /// Returns whether this matrix is a column vector.
    pub const fn is_column_vector(&self) -> bool {
        self.column_count == 1
    }

    /// Wrap this matrix in a [Vector].
    ///
    /// # Errors
    /// Will error if this matrix is not a row or column vector.
    pub const fn as_vector(&self) -> Result<Vector<T>, EvalError> {
        Vector::new(self)
    }
    /// Get all elements of it as a vector
    ///
    /// ## Panics
    /// Panics if this matrix is not a vector.
    pub fn get_all_vector_elements(&self) -> &Vec<T> {
        if self.is_vector() {
            return &self.values;
        }
        panic!("Not a vector")
    }
}

impl<T: Clone> Matrix<T> {
    /// Constructs a new `Matrix` instance filled with a default value.
    pub fn new_default(
        row_count: usize,
        column_count: usize,
        default_value: T,
    ) -> Self {
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
        let mut transposed_values =
            Vec::with_capacity(self.row_count * self.column_count);

        for column in 0..self.column_count {
            for row in 0..self.row_count {
                transposed_values.push(
                    self.values[row * self.column_count + column].clone(),
                );
            }
        }

        Matrix {
            values: transposed_values,
            row_count: self.column_count, // swap row and column counts
            column_count: self.row_count,
        }
    }
}

impl Matrix<Value> {
    /// Calculates the dot product of two matrices (treated as vectors).
    ///
    /// # Errors
    /// Returns an `Err` if:
    /// - One of the matrices isn't a vector.
    /// - The vectors do not have the same size.
    pub fn dot_product(
        &self,
        other: &Matrix<Value>,
    ) -> Result<Value, EvalError> {
        let a = self.as_vector()?;
        let b = other.as_vector()?;

        if a.get_size() != b.get_size() {
            return Err(EvalError::IncompatibleMatrixSizes {
                source: IncompatibleMatrixSizes::SameSizeVectors {
                    a: a.get_size(),
                    b: b.get_size(),
                },
            });
        }

        // Calculation
        let mut sum = Option::None;
        for i in 0..a.get_size() {
            let a_i = a.get(i);
            let b_i = b.get(i);
            let term = a_i.mul(&MulType::Implicit, b_i)?;
            sum = Some(match sum {
                Some(sum) => (sum + term)?,
                None => term,
            });
        }
        Ok(sum.expect("Empty vector"))
    }

    /// Calculates the three dimensional cross product of two matrices (treated
    /// as vectors.)
    ///
    /// # Errors
    /// Returns an `Err` if:
    /// - One of the matrices isn't a vector.
    /// - One of the matrices isn't a vector with 3 components.
    pub fn cross_product(
        &self,
        other: &Matrix<Value>,
    ) -> Result<Matrix<Value>, EvalError> {
        let a = self.as_vector()?;
        let b = other.as_vector()?;

        ///helper function that returns a
        /// [IncompatibleMatrixSizes::CrossProduct] error
        const fn size_err(m: &Vector<Value>) -> EvalError {
            EvalError::IncompatibleMatrixSizes {
                source: IncompatibleMatrixSizes::CrossProduct {
                    found_size: m.get_size(),
                },
            }
        }

        if a.get_size() != 3 {
            return Err(size_err(&a));
        }
        if b.get_size() != 3 {
            return Err(size_err(&b));
        }

        // Calculation
        let (x1, y1, z1) = (a.get(0), a.get(1), a.get(2));
        let (x2, y2, z2) = (b.get(0), b.get(1), b.get(2));
        let mt = &MulType::Implicit;
        let values = vec![
            (y1.mul(mt, z2)? - z1.mul(mt, y2)?)?,
            (z1.mul(mt, x2)? - x1.mul(mt, z2)?)?,
            (x1.mul(mt, y2)? - y1.mul(mt, x2)?)?,
        ];
        Ok(Matrix {
            values,
            row_count: 3,
            column_count: 1,
        })
    }
}

impl<Lhs> Matrix<Lhs> {
    /*
    /// Calculates the dot product of two matrices (treated as vectors).
    ///
    /// # Errors
    ///
    /// Returns an `Err` if:
    ///    - Both matrices are not vectors (i.e., more than one row or column).
    ///    - The dimensions of the matrices do not match for dot product
    ///      calculation.
    pub fn dot_product<Rhs, Res>(
        &self,
        other: &Matrix<Rhs>,
    ) -> Result<Res, &'static str>
    where
        Lhs: Mul<Rhs, Output = Res> + Clone,
        Res: AddAssign + Default,
        Rhs: Clone,
    {
        if self.row_count != other.row_count
            || self.column_count != 1
            || other.column_count != 1
        {
            return Err("Both matrices must be vectors of the same dimension.");
        }

        let mut result = Default::default();
        for i in 0..self.row_count {
            result += self.values[i].clone() * other.values[i].clone();
        }

        Ok(result)
    }
    */

    /*
    /// Calculates the cross product of two 3D vectors represented as matrices.
    ///
    /// # Errors
    ///
    /// Returns an `Err` of type `IncompatibleMatrixSizes` if:
    ///    - Either matrix does not have exactly 3 rows.
    ///    - Either matrix does not have exactly 1 column.
    pub fn cross_product<Rhs, Res>(
        &self,
        other: &Matrix<Rhs>,
    ) -> Result<Matrix<Res>, EvalError>
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
    */
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
            let term =
                (self.get(0, col).clone() * (submatrix.determinant()?))?;
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
    pub fn map<F, Res>(&self, func: F) -> Result<Matrix<Res>, EvalError>
    where
        F: Fn(&T) -> Result<Res, EvalError>,
    {
        let val: Result<_, _> = self.values.iter().map(func).collect();
        Ok(Matrix::new(val?, self.row_count, self.column_count))
    }
    /// Maps a function over all elements of the matrix without cloning.
    ///
    /// # Errors
    ///
    /// Returns an `Err` of type `EvalError` if the provided function `func`
    /// returns an error for any of the matrix elements.
    pub fn map_owned<F, Res>(self, func: F) -> Result<Matrix<Res>, EvalError>
    where
        F: Fn(T) -> Result<Res, EvalError>,
    {
        let val: Result<_, _> = self.values.into_iter().map(func).collect();
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
    pub fn pair_map<F, Res, Out>(
        &self,
        rhs: Matrix<Out>,
        func: F,
    ) -> Result<Matrix<Res>, EvalError>
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

#[cfg(test)]
impl Matrix<MathExpr> {
    pub fn zero(rows: usize, cols: usize) -> Matrix<MathExpr> {
        let values = vec![
            MathExpr::Term(Term::Factor(Factor::Constant(0.0)));
            rows * cols
        ];
        Self {
            values,
            row_count: rows,
            column_count: cols,
        }
    }
}

impl Matrix<Value> {
    ///Normal matrix multiplication
    pub fn matrix_mul(
        &self,
        rhs: &Matrix<Value>,
    ) -> Result<Matrix<Value>, EvalError> {
        if self.column_count != rhs.row_count {
            return Err(IncompatibleMatrixSizes::Row {
                expected: self.column_count,
                found: rhs.row_count,
            }
            .into());
        }

        let mut result = Matrix::new_default(
            self.row_count,
            rhs.column_count,
            Value::Scalar(0.0),
        );
        for i in 0..self.row_count {
            for j in 0..rhs.column_count {
                let mut sum = Option::None;
                for k in 0..self.column_count {
                    let a = self.get(i, k);
                    let b = rhs.get(k, j);
                    let term = (a.mul(&MulType::Implicit, b))?;
                    sum = Some(match sum {
                        Some(prev) => (prev + term)?,
                        None => term,
                    });
                }
                result.set(i, j, sum.expect("Nothing was multiplied."));
            }
        }

        Ok(result)
    }
}

/*
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
 */

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

impl<Lhs: Clone + Mul<Value, Output = Result<Value, EvalError>>> Mul<Value>
    for Matrix<Lhs>
{
    type Output = Result<Matrix<Value>, EvalError>;

    fn mul(self, rhs: Value) -> Self::Output {
        // Multiply matrix components by self
        self.map(|val| val.clone() * rhs.clone())
    }
}
impl Mul<f64> for &Matrix<Value> {
    type Output = Result<Matrix<Value>, EvalError>;

    fn mul(self, rhs: f64) -> Self::Output {
        // Multiply matrix components by self
        self.map(|val| val * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::Matrix;
    use crate::prelude::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn matrix_scalar_value_addition() {
        let a = Matrix::new_default(2, 3, Value::Scalar(1.0));
        let b = Matrix::new_default(2, 3, Value::Scalar(2.0));
        let c = Matrix::new_default(2, 3, Value::Scalar(3.0));

        assert_eq!((a + b).unwrap(), c);
    }

    #[test]
    fn matrix_scalar_value_subtraction() {
        let a = Matrix::new_default(2, 3, Value::Scalar(3.0));
        let b = Matrix::new_default(2, 3, Value::Scalar(1.0));
        let c = Matrix::new_default(2, 3, Value::Scalar(2.0));

        assert_eq!((a - b).unwrap(), c);
    }

    #[test]
    fn matrix_2x2_scalar_value_multiplication() {
        let mut a = Matrix::new_default(2, 2, Value::Scalar(0.0));
        a.set(0, 0, Value::Scalar(1.0));
        a.set(0, 1, Value::Scalar(2.0));
        a.set(1, 0, Value::Scalar(3.0));
        a.set(1, 1, Value::Scalar(4.0));
        let mut b = Matrix::new_default(2, 2, Value::Scalar(0.0));
        b.set(0, 0, Value::Scalar(5.0));
        b.set(0, 1, Value::Scalar(6.0));
        b.set(1, 0, Value::Scalar(7.0));
        b.set(1, 1, Value::Scalar(8.0));
        let mut c = Matrix::new_default(2, 2, Value::Scalar(0.0));
        c.set(0, 0, Value::Scalar(19.0));
        c.set(0, 1, Value::Scalar(22.0));
        c.set(1, 0, Value::Scalar(43.0));
        c.set(1, 1, Value::Scalar(50.0));

        assert_eq!((a.matrix_mul(&b)).unwrap(), c);
    }

    #[test]
    fn matrix_3x2_times_2x1_scalar_value_multiplication() {
        let mut a = Matrix::new_default(3, 2, Value::Scalar(0.0));
        a.set(0, 0, Value::Scalar(1.0));
        a.set(0, 1, Value::Scalar(2.0));
        a.set(1, 0, Value::Scalar(3.0));
        a.set(1, 1, Value::Scalar(4.0));
        a.set(2, 0, Value::Scalar(5.0));
        a.set(2, 1, Value::Scalar(6.0));
        let mut b = Matrix::new_default(2, 1, Value::Scalar(0.0));
        b.set(0, 0, Value::Scalar(7.0));
        b.set(1, 0, Value::Scalar(8.0));
        let mut c = Matrix::new_default(3, 1, Value::Scalar(0.0));
        c.set(0, 0, Value::Scalar(23.0));
        c.set(1, 0, Value::Scalar(53.0));
        c.set(2, 0, Value::Scalar(83.0));

        assert_eq!((a.matrix_mul(&b)).unwrap(), c);
    }

    #[test]
    fn dot_product_row_column_vectors() {
        let mut a = Matrix::new_default(1, 3, Value::Scalar(0.0));
        a.set(0, 0, Value::Scalar(1.0));
        a.set(0, 1, Value::Scalar(2.0));
        a.set(0, 2, Value::Scalar(3.0));
        let mut b = Matrix::new_default(3, 1, Value::Scalar(0.0));
        b.set(0, 0, Value::Scalar(4.0));
        b.set(1, 0, Value::Scalar(5.0));
        b.set(2, 0, Value::Scalar(6.0));

        assert_eq!(a.dot_product(&b).unwrap(), Value::Scalar(32.0));
        assert_eq!(b.dot_product(&a).unwrap(), Value::Scalar(32.0));
    }

    #[test]
    fn dot_product_row_vectors() {
        let mut a = Matrix::new_default(1, 3, Value::Scalar(0.0));
        a.set(0, 0, Value::Scalar(1.0));
        a.set(0, 1, Value::Scalar(2.0));
        a.set(0, 2, Value::Scalar(3.0));
        let mut b = Matrix::new_default(1, 3, Value::Scalar(0.0));
        b.set(0, 1, Value::Scalar(5.0));
        b.set(0, 0, Value::Scalar(4.0));
        b.set(0, 2, Value::Scalar(6.0));

        assert_eq!(a.dot_product(&b).unwrap(), Value::Scalar(32.0));
        assert_eq!(b.dot_product(&a).unwrap(), Value::Scalar(32.0));
    }

    #[test]
    fn dot_product_column_vectors() {
        let mut a = Matrix::new_default(3, 1, Value::Scalar(0.0));
        a.set(0, 0, Value::Scalar(1.0));
        a.set(1, 0, Value::Scalar(2.0));
        a.set(2, 0, Value::Scalar(3.0));
        let mut b = Matrix::new_default(3, 1, Value::Scalar(0.0));
        b.set(0, 0, Value::Scalar(4.0));
        b.set(1, 0, Value::Scalar(5.0));
        b.set(2, 0, Value::Scalar(6.0));

        assert_eq!(a.dot_product(&b).unwrap(), Value::Scalar(32.0));
        assert_eq!(b.dot_product(&a).unwrap(), Value::Scalar(32.0));
    }

    #[test]
    fn cross_product() {
        let mut x = Matrix::new_default(3, 1, Value::Scalar(0.0));
        x.set(0, 0, Value::Scalar(1.0));
        x.set(1, 0, Value::Scalar(0.0));
        x.set(2, 0, Value::Scalar(0.0));
        let mut y = Matrix::new_default(3, 1, Value::Scalar(0.0));
        y.set(0, 0, Value::Scalar(0.0));
        y.set(1, 0, Value::Scalar(1.0));
        y.set(2, 0, Value::Scalar(0.0));
        let mut z = Matrix::new_default(3, 1, Value::Scalar(0.0));
        z.set(0, 0, Value::Scalar(0.0));
        z.set(1, 0, Value::Scalar(0.0));
        z.set(2, 0, Value::Scalar(1.0));

        assert_eq!(x.cross_product(&y).unwrap(), z);
        assert_eq!(y.cross_product(&x).unwrap(), (&z * -1.0).unwrap());
        assert_eq!(z.cross_product(&x).unwrap(), y);
        assert_eq!(x.cross_product(&z).unwrap(), (&y * -1.0).unwrap());
        assert_eq!(y.cross_product(&z).unwrap(), x);
        assert_eq!(z.cross_product(&y).unwrap(), (&x * -1.0).unwrap());
    }
}
