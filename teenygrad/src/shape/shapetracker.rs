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
use std::{iter::zip, ops::Mul, vec};

use super::symbolic::Node;

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

pub fn filter_strides(shape: &[isize], strides: &[isize]) -> Vec<isize> {
    debug_assert!(shape.len() == strides.len());

    izip!(strides, shape)
        .map(|(s, shp)| if *shp != 1 { *s } else { 0 })
        .collect()
}

// class ViewInternal(NamedTuple):
//   shape:Tuple[int, ...]
//   strides:Tuple[int, ...]
//   offset:int
//   mask:Optional[Tuple[Tuple[int, int]]]
//   contiguous:bool
//   shape_strides:Tuple[Tuple[int, int], ...]

pub struct View {
    shape: Vec<isize>,
    strides: Vec<isize>,
    offset: isize,
    mask: Option<Vec<(isize, isize)>>,
    contiguous: bool,
    shape_strides: Vec<(isize, isize)>,
}

impl View {
    pub fn new(
        shape: &[isize],
        strides: Option<&[isize]>,
        offset: isize,
        mask: Option<&[(isize, isize)]>,
    ) -> Self {
        let strides_from_shape = strides_for_shape(shape);
        let filtered_strides = match strides {
            Some(s) => filter_strides(shape, s),
            None => strides_from_shape,
        };
        let contiguous = offset == 0 && mask.is_none() && is_contiguous(shape, &filtered_strides);
        let shape_strides = to_shape_strides(shape, &filtered_strides);
        let mask: Option<Vec<(isize, isize)>> = mask.map(|mask| mask.into());

        View {
            shape: shape.into(),
            strides: filtered_strides,
            offset,
            mask,
            contiguous,
            shape_strides,
        }
    }

    pub fn expr_node_mask(&self, idx: isize, valid: Option<Node>) -> Node {
        let mut expr = match valid {
            Some(v) => vec![v],
            None => Vec::new(),
        };

        if let Some(mask) = &self.mask {
            let mut acc = 1;
            for (ns, (x, y)) in self.shape.iter().zip(mask.iter()).rev() {
                let base = (idx / acc) % ns;
                expr.push(Node::new_num(base).ge(*x));
                expr.push(Node::new_num(base).lt(*y));
                acc *= ns;
            }
        }

        Node::new_ands(expr.as_ref())
    }

    pub fn expr_node(&self, idx: Option<Node>) -> Node {
        let idx = match idx {
            None => Node::new_var("idx", 0, self.shape.iter().product::<isize>() - 1),
            Some(idx) => idx,
        };
        let mut result = if self.offset != 0 {
            vec![Node::new_num(self.offset)]
        } else {
            vec![]
        };
        let mut acc: isize = 1;
        for (d, s) in self.shape_strides.iter().rev() {
            result.push(idx.clone().floordiv(acc, None).modulus(*d).mul(*s));
            acc *= d;
        }

        Node::new_sum(result.as_ref())
    }

    pub fn expr_idxs(&self, idxs: &[Node]) -> Node {
        debug_assert!(idxs.len() == self.shape.len());

        let mut result = vec![Node::new_num(self.offset)];

        for (idx, (sh, st)) in idxs.iter().zip(self.shape.iter().zip(self.strides.iter())) {
            if *sh != 1 && *st != 0 {
                result.push(idx.clone().mul(*st));
            }
        }

        Node::new_sum(result.as_ref())
    }
}

// @functools.lru_cache(maxsize=None)
// def idxs_to_idx(shape:Tuple[int, ...], idxs) -> Node:
//   assert len(idxs) == len(shape), "need an idx for all dimensions"
//   acc = 1
//   ret = []
//   for tidx,d in reversed(list(zip(idxs, shape))):
//     ret.append(tidx * acc)
//     acc *= d
//   return Variable.sum(ret)

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

    #[test]
    fn test_filter_strides() {
        assert_eq!(filter_strides(&[1, 1, 1], &[0, 0, 0]), vec![0, 0, 0]);
        assert_eq!(filter_strides(&[2, 3, 4], &[6, 3, 1]), vec![6, 3, 1]);
        assert_eq!(
            filter_strides(&[2, 3, 4, 5], &[24, 12, 4, 1]),
            vec![24, 12, 4, 1]
        );
    }
}
