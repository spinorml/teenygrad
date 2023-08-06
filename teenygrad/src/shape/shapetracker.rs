/*
 * Copyright (c) 2023 SpinorML
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use itertools::izip;
use std::iter::zip;

pub enum MovementOps {
    Reshape,
    Permute,
    Expand,
    Pad,
    Shrink,
    Stride,
}

pub fn to_shape_strides(shape: &[isize], strides: &[isize]) -> Vec<(isize, isize)> {
    debug_assert!(shape.len() == strides.len());
    let mut result: Vec<(isize, isize)> = Vec::new();
    if !shape.is_empty() {
        result.push((shape[0], strides[0]));
    }

    for i in 1..shape.len() {
        if (strides[i] != 0 && result.last().unwrap().1 == shape[i] * strides[i])
            || result.last().unwrap().0 == 1
            || (strides[i] == 0 && result.last().unwrap().1 == 0)
        {
            result.last_mut().unwrap().0 *= shape[i];
            result.last_mut().unwrap().1 = strides[i];
        } else {
            result.push((shape[i], strides[i]));
        }
    }

    result
}

pub fn strides_for_shape(shape: &[isize]) -> Vec<isize> {
    let mut strides: Vec<isize> = Vec::new();
    if !shape.is_empty() {
        strides.push(1);
    }

    for d in shape.iter().rev().skip(1) {
        strides.insert(0, d * strides[0]);
    }

    zip(strides, shape)
        .map(|(st, s)| if *s != 1 { st } else { 0 })
        .collect()
}

pub fn is_contiguous(shape: &[isize], strides: &[isize]) -> bool {
    debug_assert!(shape.len() == strides.len());

    izip!(shape, strides, &strides_for_shape(shape)).all(|(s, s1, s2)| *s1 == *s2 || *s == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_shape_strides() {
        assert_eq!(to_shape_strides(&[1, 1, 1], &[0, 0, 0]), vec![(1, 0)]);
        assert_eq!(
            to_shape_strides(&[2, 3, 4], &[6, 3, 1]),
            vec![(2, 6), (3, 3), (4, 1)]
        );
        assert_eq!(
            to_shape_strides(&[2, 3, 1], &[6, 3, 0]),
            vec![(2, 6), (3, 3), (1, 0)]
        );
        assert_eq!(
            to_shape_strides(&[2, 3, 4, 5], &[24, 12, 4, 1]),
            vec![(2, 24), (3, 12), (4, 4), (5, 1)]
        );
    }

    #[test]
    fn test_strides_for_shape() {
        assert_eq!(strides_for_shape(&[1, 1, 1]), vec![0, 0, 0]);
        assert_eq!(strides_for_shape(&[2, 3, 4]), vec![6, 3, 1]);
        assert_eq!(strides_for_shape(&[2, 3, 1]), vec![6, 3, 0]);
        assert_eq!(strides_for_shape(&[2, 3, 4, 5]), vec![24, 12, 4, 1]);
    }

    #[test]
    fn test_is_contiguous() {
        assert!(is_contiguous(&[1, 1, 1], &[0, 0, 0]));
        assert!(is_contiguous(&[2, 3, 4], &[6, 3, 1]));
        assert!(is_contiguous(&[2, 3, 4, 5], &[24, 12, 4, 1]));
        assert!(!is_contiguous(&[2, 3, 4, 5], &[24, 12, 4, 2]));
        assert!(!is_contiguous(&[2, 3, 4, 5], &[24, 12, 4, 0]));
    }
}
