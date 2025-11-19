use std::iter::Sum;
use std::ops::*;
// ============================================================================
// This code is part of RPB.
// ----------------------------------------------------------------------------
// MIT License
//
// Copyright (c) 2023-present Javad Abdi, Mark C. Jeffrey
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
// ============================================================================

use num_traits::Float;
use std::fmt::{Debug, Display, LowerExp};
use std::str::FromStr;

use crate::common::io::fmt_f64;

const PI: f64 = 3.14159;
pub trait PointToVec {
    type Vec;
}

// *************************************************************
//    POINTS AND VECTORS
// *************************************************************

#[derive(Copy, Clone)]
pub struct Vector<const D: usize, T>([T; D]);

impl<const D: usize, T> Vector<D, T> {
    pub fn new(coords: [T; D]) -> Self {
        Self(coords)
    }
}

impl<const D: usize, T> Index<usize> for Vector<D, T> {
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        &self.0[idx]
    }
}

impl<T: Float> Vector<2, T> {
    pub fn cross(&self, v: Vector<2, T>) -> T {
        self[0] * v[1] - self[1] * v[0]
    }
}

impl<const D: usize, T> Vector<D, T>
where
    T: Mul<Output = T> + Sum + Copy,
{
    pub fn length_squared(&self) -> T {
        self.0.iter().map(|x| *x * *x).sum()
    }
}

impl<const D: usize, T: Float + Sum> Vector<D, T> {
    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }
    pub fn dot(&self, v: Vector<D, T>) -> T {
        self.0
            .iter()
            .zip(v.0.iter())
            .map(|(x, y)| (*x) * (*y))
            .sum()
    }
}

impl<const D: usize, T: Float> Mul<T> for Vector<D, T> {
    type Output = Self;
    fn mul(self, other: T) -> Self {
        Self(self.0.map(|x| x * other))
    }
}

impl<const D: usize, T: Float> Div<T> for Vector<D, T> {
    type Output = Self;
    fn div(self, other: T) -> Self {
        Self(self.0.map(|x| x / other))
    }
}

impl<const D: usize, T: Float> Default for Vector<D, T> {
    fn default() -> Self {
        Self([T::zero(); D])
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Point<const D: usize, T>([T; D]);

impl<const D: usize, T> Index<usize> for Point<D, T> {
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        &self.0[idx]
    }
}

impl<const D: usize, T> IndexMut<usize> for Point<D, T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.0[idx]
    }
}

impl<const D: usize, T> PointToVec for Point<D, T> {
    type Vec = Vector<D, T>;
}

impl<const D: usize, T> Point<D, T> {
    pub fn new(coords: [T; D]) -> Self {
        Self(coords)
    }
}

impl<const D: usize, T: Float> Default for Point<D, T> {
    fn default() -> Self {
        Self::new([T::zero(); D])
    }
}

impl<const D: usize, T: Float> Add<Vector<D, T>> for Point<D, T> {
    type Output = Self;
    fn add(self, other: Vector<D, T>) -> Self {
        Self(std::array::from_fn(|i| self[i] + other[i]))
    }
}

impl<const D: usize, T, U> Sub for Point<D, T>
where
    T: Sub<Output = U> + Copy,
{
    type Output = Vector<D, U>;
    fn sub(self, other: Self) -> Vector<D, U> {
        Vector::new(std::array::from_fn(|i| self[i] - other[i]))
    }
}

impl<const D: usize, T: Float + FromStr> FromStr for Point<D, T>
where
    <T as std::str::FromStr>::Err: Debug,
{
    type Err = ParsePointError<D>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(
            s.trim()
                .split_whitespace()
                .map(|x| x.parse())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| ParsePointError)?
                .try_into()
                .map_err(|_| ParsePointError)?,
        ))
    }
}

impl<const D: usize, T: Float + Display + LowerExp> Display for Point<D, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .map(|x| fmt_f64(x.to_f64().unwrap(), 11, 2))
                .join(" ")
        )
    }
}

pub struct ParsePointError<const D: usize>;

impl<const D: usize> Display for ParsePointError<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Can not parse as Point{D}d.")
    }
}

impl<const D: usize> Debug for ParsePointError<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{{ file: {}, line: {} }}: can not parse as Point{D}d.",
            file!(),
            line!()
        )
    }
}

impl<T: Float> Vector<3, T> {
    // Returns the vector result of the cross product
    pub fn cross(self, other: Self) -> Self {
        Self::new([
            self[1] * other[2] - self[2] * other[1],
            self[2] * other[0] - self[0] * other[2],
            self[0] * other[1] - self[1] * other[0],
        ])
    }
}

//    POINTS AND VECTORS
// *************************************************************

// *************************************************************
//    TRIANGLES AND RAYS
// *************************************************************

pub type Tri = [i32; 3];

#[derive(Clone)]
pub struct Triangles<P> {
    pub p: Vec<P>,
    pub t: Vec<Tri>,
}

impl<P> Triangles<P> {
    pub fn new(p: Vec<P>, t: Vec<Tri>) -> Self {
        Self { p, t }
    }
    pub fn num_points(&self) -> usize {
        self.p.len()
    }
    pub fn num_triangles(&self) -> usize {
        self.t.len()
    }
}

#[derive(Copy, Clone)]
pub struct Ray<P>
where
    P: PointToVec,
{
    pub o: P,
    pub d: P::Vec,
}

impl<P> Ray<P>
where
    P: PointToVec,
{
    pub fn new(o: P, d: P::Vec) -> Self {
        Self { o, d }
    }
}

#[inline(always)]
pub fn angle<T: Float + Sum>(a: Point<2, T>, b: Point<2, T>, c: Point<2, T>) -> T {
    let (ba, ca) = (b - a, c - a);
    let (lba, lca) = (ba.length(), ca.length());
    T::from(180.0 / PI * (ba.dot(ca) / (lba * lca)).to_f64().unwrap().acos()).unwrap()
}

#[inline(always)]
pub fn min_angle_check<T: Float + Sum>(
    a: Point<2, T>,
    b: Point<2, T>,
    c: Point<2, T>,
    angle: T,
) -> bool {
    let (ba, ca, cb) = (b - a, c - a, c - b);
    let (lba, lca, lcb) = (ba.length(), ca.length(), cb.length());
    let co = T::from((angle.to_f64().unwrap() * PI / 180.0).cos()).unwrap();

    ba.dot(ca) / (lba * lca) > co || ca.dot(cb) / (lca * lcb) > co || -ba.dot(cb) / (lba * lcb) > co
}

#[inline(always)]
pub fn triangle_circumcenter<T: Float + Sum>(
    a: Point<2, T>,
    b: Point<2, T>,
    c: Point<2, T>,
) -> Point<2, T> {
    let (v1, v2) = (b - a, c - a);
    let (v11, v22) = (v1 * v2.dot(v2), v2 * v1.dot(v1));

    a + Vector::new([v22[1] - v11[1], v11[0] - v22[0]]) / (T::from(2.0).unwrap() * v1.cross(v2))
}

//    TRIANGLES AND RAYS
// *************************************************************

// *************************************************************
//    GEOMETRY
// *************************************************************

#[inline(always)]
pub fn tri_area<T: Float>(a: Point<2, T>, b: Point<2, T>, c: Point<2, T>) -> T {
    (b - a).cross(c - a)
}

#[inline(always)]
pub fn counter_clock_wise<T: Float>(a: Point<2, T>, b: Point<2, T>, c: Point<2, T>) -> bool {
    (b - a).cross(c - a) > T::zero()
}

#[inline(always)]
pub fn on_parabola<T: Float>(v: Vector<2, T>) -> Vector<3, T> {
    Vector::new([v[0], v[1], v[0] * v[0] + v[1] * v[1]])
}

#[inline(always)]
pub fn in_circle<T: Float + Sum>(
    a: Point<2, T>,
    b: Point<2, T>,
    c: Point<2, T>,
    d: Point<2, T>,
) -> bool {
    let ad = on_parabola(a - d);
    let bd = on_parabola(b - d);
    let cd = on_parabola(c - d);
    (ad.cross(bd)).dot(cd) > T::zero()
}
