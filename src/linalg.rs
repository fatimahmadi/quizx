// QuiZX - Rust library for quantum circuit rewriting and optimisation
//         using the ZX-calculus
// Copyright (C) 2021 - Aleks Kissinger
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Matrices and linear algebra over F2

use std::fmt;
use std::cmp::min;
use rustc_hash::FxHashMap;

/// A type for matrices over F2
#[derive(PartialEq,Eq,Clone,Debug)]
pub struct Mat2 {
    d: Vec<Vec<u32>>
}

pub trait RowColOps {
    /// Add r0 to r1
    fn row_add(&mut self, r0: usize, r1: usize);
    /// Add c0 to c1
    fn col_add(&mut self, c0: usize, c1: usize);
    /// Swap r0 and r1
    fn row_swap(&mut self, r0: usize, r1: usize);
    /// Swap c0 and c1
    fn col_swap(&mut self, c0: usize, c1: usize);
}

/// Make unit implement RowColOps to allow optional args
impl RowColOps for () {
    fn row_add(&mut self, _: usize, _: usize) {}
    fn col_add(&mut self, _: usize, _: usize) {}
    fn row_swap(&mut self, _: usize, _: usize) {}
    fn col_swap(&mut self, _: usize, _: usize) {}
}

impl Mat2 {
    pub fn new(d: Vec<Vec<u32>>) -> Mat2 {
        Mat2 { d }
    }

    /// Build a matrix with the given number of rows and columns. Place a 1
    /// wherever f(i,j) is true.
    pub fn build<F>(rows: usize, cols: usize, f: F) -> Mat2
        where F: Fn(usize, usize) -> bool
    {
        Mat2 {
            d: (0..rows).map(|x| (0..cols).map(|y|
                   if f(x, y) { 1 } else { 0 })
               .collect()).collect()
        }
    }

    /// A matrix full of zeros
    pub fn zeros(rows: usize, cols: usize) -> Mat2 {
        Mat2::build(rows, cols, |_,_| false)
    }

    /// A matrix full of ones
    pub fn ones(rows: usize, cols: usize) -> Mat2 {
        Mat2::build(rows, cols, |_,_| true)
    }

    /// The identity matrix of a given size
    pub fn id(dim: usize) -> Mat2 {
        Mat2::build(dim, dim, |x,y| x == y)
    }

    /// A column vector with a single 1 at the given index
    pub fn unit_vector(dim: usize, i: usize) -> Mat2 {
        Mat2::build(dim, 1, |x,_| x == i)
    }

    pub fn num_rows(&self) -> usize {
        self.d.len()
    }

    pub fn num_cols(&self) -> usize {
        if self.d.len() > 0 { self.d[0].len() }
        else { 0 }
    }

    /// Return the transpose as a copy
    pub fn transpose(&self) -> Mat2 {
        Mat2::build(self.num_cols(), self.num_rows(),
                    |i,j| self[j][i] == 1)
    }

    /// Main function for computing the echelon form.
    ///
    /// Returns the number of non-zero rows in the result, i.e.
    /// the rank of the matrix.
    ///
    /// The parameter 'full_reduce' determines whether to compute the full row-reduced form,
    /// useful e.g. for matrix inversion and CNOT circuit synthesis.
    ///
    /// The parameter 'blocksize' gives the size of the blocks in a block matrix for
    /// performing Patel/Markov/Hayes optimization, see:
    ///
    /// K. Patel, I. Markov, J. Hayes. Optimal Synthesis of Linear Reversible
    /// Circuits. QIC 2008
    ///
    /// If blocksize is given as self.cols(), then
    /// this is equivalent to just eliminating duplicate rows before doing normal
    /// Gaussian elimination.
    ///
    /// Contains two convenience parameters for saving the primitive row operations. Suppose
    /// the row-reduced form of m is computed as:
    ///
    /// g * m = m'
    ///
    /// Then, x --> g * x and y --> y * g^-1.
    ///
    /// Note x and y need not be matrices. x can be any object that implements the method
    /// row_add(), and y any object that implements col_add().
    fn gauss_helper<S,T>(&mut self, full_reduce: bool, blocksize: usize,
                      x: &mut S, y: &mut T, pivot_cols: &mut Vec<usize>) -> usize
        where S: RowColOps, T: RowColOps
    {
        let rows = self.num_rows();
        let cols = self.num_cols();
        let mut pivot_row = 0;

        let num_blocks =
            if cols % blocksize == 0 { cols / blocksize }
            else { (cols / blocksize) + 1 };

        for sec in 0..num_blocks {
            let i0 = sec * blocksize;
            let i1 = min(cols, (sec+1) * blocksize);

            let mut chunks: FxHashMap<Vec<u32>,usize> =
                FxHashMap::default();
            for r in pivot_row..rows {
                let ch = self.d[r][i0..i1].to_vec();
                if ch.iter().all(|x| *x == 0) { continue; }
                if let Some(r1) = chunks.get(&ch) {
                    self.row_add(*r1, r);
                    x.row_add(*r1, r);
                    y.col_add(r, *r1);
                } else {
                    chunks.insert(ch, r);
                }
            }

            for p in i0..i1 {
                for r0 in pivot_row..rows {
                    if self.d[r0][p] != 0 {
                        if r0 != pivot_row {
                            self.row_add(r0, pivot_row);
                            x.row_add(r0, pivot_row);
                            y.col_add(pivot_row, r0);
                        }

                        for r1 in pivot_row+1..rows {
                            if pivot_row != r1 && self.d[r1][p] != 0 {
                                self.row_add(pivot_row, r1);
                                x.row_add(pivot_row, r1);
                                y.col_add(r1, pivot_row);
                            }
                        }
                        pivot_cols.push(p);
                        pivot_row += 1;
                        break;
                    }
                }
            }
        }

        let rank = pivot_row;

        if full_reduce {
            pivot_row -= 1;
            let mut pivot_cols1 = pivot_cols.clone();

            for sec in (0..num_blocks).rev() {
                let i0 = sec * blocksize;
                let i1 = min(cols, (sec+1) * blocksize);

                let mut chunks: FxHashMap<Vec<u32>,usize> =
                    FxHashMap::default();
                for r in (0..=pivot_row).rev() {
                    let ch = self.d[r][i0..i1].to_vec();
                    if ch.iter().all(|x| *x == 0) { continue; }
                    if let Some(r1) = chunks.get(&ch) {
                        self.row_add(*r1, r);
                        x.row_add(*r1, r);
                        y.col_add(r, *r1);
                    } else {
                        chunks.insert(ch, r);
                    }
                }

                loop {
                    if let Some(pcol) = pivot_cols1.pop() {
                        if i0 > pcol || pcol >= i1 { break; }
                        for r in 0..pivot_row {
                            if self.d[r][pcol] != 0 {
                                self.row_add(pivot_row, r);
                                x.row_add(pivot_row, r);
                                y.col_add(r, pivot_row);
                            }
                        }
                        if pivot_row > 0 { pivot_row -= 1; }
                    } else {
                        break;
                    }
                }
            }
        }
        rank
    }

    pub fn gauss(&mut self, full_reduce: bool) -> usize {
        self.gauss_helper(full_reduce, 3, &mut (), &mut (), &mut vec![])
    }

    pub fn gauss_aux(&mut self, full_reduce: bool, blocksize: usize, x: &mut impl RowColOps) -> usize {
        self.gauss_helper(full_reduce, blocksize, x, &mut (), &mut vec![])
    }

    pub fn rank(&self) -> usize {
        let mut m = self.clone();
        m.gauss(false)
    }

    pub fn inverse(&self) -> Option<Mat2> {
        if self.num_rows() != self.num_cols() {
            return None;
        }

        let mut m = self.clone();
        let mut inv = Mat2::id(self.num_rows());
        let rank = m.gauss_helper(true, 3, &mut inv, &mut (), &mut vec![]);

        if rank < self.num_rows() {
            None
        } else {
            Some(inv)
        }
    }
}

impl RowColOps for Mat2 {
    fn row_add(&mut self, r0: usize, r1: usize) {
        for i in 0..self.num_cols() {
            self.d[r1][i] = (self.d[r0][i] + self.d[r1][i]) % 2;
        }
    }

    fn col_add(&mut self, c0: usize, c1: usize) {
        for i in 0..self.num_rows() {
            self.d[i][c1] = (self.d[i][c0] + self.d[i][c1]) % 2;
        }
    }

    fn row_swap(&mut self, r0: usize, r1: usize) {
        self.d.swap(r0, r1);
    }

    fn col_swap(&mut self, c0: usize, c1: usize) {
        for i in 0..self.num_rows() {
            self.d[i].swap(c0, c1);
        }
    }
}

impl fmt::Display for Mat2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.d {
            write!(f, "[ ")?;
            for x in row { write!(f, "{} ", x)?; }
            write!(f, "]\n")?;
        }
        Ok(())
    }
}

impl std::ops::Index<(usize,usize)> for Mat2 {
    type Output = u32;
    fn index(&self, idx: (usize,usize)) -> &Self::Output { &self.d[idx.0][idx.1] }
}

impl std::ops::IndexMut<(usize,usize)> for Mat2 {
    fn index_mut(&mut self, idx: (usize,usize)) -> &mut Self::Output { &mut self.d[idx.0][idx.1] }
}

impl std::ops::Index<usize> for Mat2 {
    type Output = Vec<u32>;
    fn index(&self, idx: usize) -> &Self::Output { &self.d[idx] }
}

impl std::ops::IndexMut<usize> for Mat2 {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output { &mut self.d[idx] }
}

impl<'a, 'b> std::ops::Mul<&'b Mat2> for &'a Mat2 {
    type Output = Mat2;

    fn mul(self, rhs: &Mat2) -> Self::Output {
        if self.num_cols() != rhs.num_rows() {
            panic!("Cannot multiply matrices with mismatched dimensions.");
        }

        Mat2 {
            d: (0..self.num_rows()).map(|x|
                 (0..rhs.num_cols()).map(|y|
                   (0..self.num_cols()).map(|i|
                     self.d[x][i] * rhs.d[i][y]
                   ).sum::<u32>() % 2
                 ).collect()
               ).collect()
        }
    }
}

impl<'a> std::ops::Mul<Mat2> for &'a Mat2 {
    type Output = Mat2;
    fn mul(self, rhs: Mat2) -> Self::Output { self * &rhs } }
impl<'a> std::ops::Mul<&'a Mat2> for Mat2 {
    type Output = Mat2;
    fn mul(self, rhs: &Mat2) -> Self::Output { &self * rhs } }
impl std::ops::Mul<Mat2> for Mat2 {
    type Output = Mat2;
    fn mul(self, rhs: Mat2) -> Self::Output { &self * &rhs } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat_mul() {
        let v = Mat2::new(vec![
            vec![1, 0, 1, 0],
            vec![1, 1, 1, 1],
            vec![0, 0, 1, 1],
        ]);

        let w = Mat2::new(vec![
            vec![1, 1],
            vec![1, 0],
            vec![0, 0],
            vec![0, 1],
        ]);

        let u = Mat2::new(vec![
            vec![1, 1],
            vec![0, 0],
            vec![0, 1],
        ]);

        assert_eq!(&v * &w, u);
    }

    #[test]
    fn transpose() {
        let v = Mat2::new(vec![
            vec![1, 0, 1, 0],
            vec![1, 1, 1, 1],
            vec![0, 0, 1, 1],
        ]);

        let vt = Mat2::new(vec![
            vec![1, 1, 0],
            vec![0, 1, 0],
            vec![1, 1, 1],
            vec![0, 1, 1],
        ]);

        assert_eq!(v.transpose(), vt);
    }

    #[test]
    fn unit_vecs() {
        let v = Mat2::new(vec![
            vec![1, 0, 1, 0],
            vec![1, 1, 1, 1],
            vec![0, 0, 1, 1],
        ]);

        let c0 = Mat2::new(vec![vec![1,1,0]]).transpose();
        let c1 = Mat2::new(vec![vec![0,1,0]]).transpose();
        let c2 = Mat2::new(vec![vec![1,1,1]]).transpose();
        let c3 = Mat2::new(vec![vec![0,1,1]]).transpose();

        assert_eq!(&v * Mat2::unit_vector(4, 0), c0);
        assert_eq!(&v * Mat2::unit_vector(4, 1), c1);
        assert_eq!(&v * Mat2::unit_vector(4, 2), c2);
        assert_eq!(&v * Mat2::unit_vector(4, 3), c3);
    }

    #[test]
    fn row_ops() {
        let mut v = Mat2::new(vec![
            vec![1, 0, 1, 0],
            vec![1, 1, 1, 1],
            vec![0, 0, 1, 1],
        ]);

        let w1 = Mat2::new(vec![
            vec![1, 0, 1, 0],
            vec![1, 1, 1, 1],
            vec![1, 1, 0, 0],
        ]);

        let w2 = Mat2::new(vec![
            vec![1, 1, 1, 1],
            vec![1, 0, 1, 0],
            vec![1, 1, 0, 0],
        ]);

        v.row_add(1, 2);
        assert_eq!(v, w1);
        v.row_swap(0, 1);
        assert_eq!(v, w2);
    }

    #[test]
    fn col_ops() {
        let mut v = Mat2::new(vec![
            vec![1, 0, 1, 0],
            vec![1, 1, 1, 1],
            vec![0, 0, 1, 1],
        ]);

        let w1 = Mat2::new(vec![
            vec![1, 1, 1, 0],
            vec![1, 0, 1, 1],
            vec![0, 1, 1, 1],
        ]);

        let w2 = Mat2::new(vec![
            vec![0, 1, 1, 1],
            vec![1, 0, 1, 1],
            vec![1, 1, 1, 0],
        ]);

        v.col_add(2, 1);
        assert_eq!(v, w1);
        v.col_swap(0, 3);
        assert_eq!(v, w2);
    }

    #[test]
    fn ranks() {
        let v = Mat2::new(vec![
            vec![1, 0, 1, 0],
            vec![1, 1, 1, 1],
            vec![0, 0, 1, 1],
        ]);
        assert_eq!(v.rank(), 3);

        let v = Mat2::new(vec![
            vec![1, 0, 1, 0],
            vec![1, 1, 1, 1],
            vec![0, 1, 0, 1],
        ]);
        assert_eq!(v.rank(), 2);
    }

    #[test]
    fn inv() {
        let v = Mat2::new(vec![
            vec![1, 1, 1],
            vec![0, 1, 1],
            vec![0, 0, 1],
        ]);
        assert_eq!(v.rank(), 3);

        let vi = v.inverse().expect("v should be invertible");
        assert_eq!(&v * &vi, Mat2::id(3));
        assert_eq!(&vi * &v, Mat2::id(3));

        let vi_exp = Mat2::new(vec![
            vec![1, 1, 0],
            vec![0, 1, 1],
            vec![0, 0, 1],
        ]);
        assert_eq!(vi_exp, vi);
    }
}

