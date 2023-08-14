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

use core::panic;
use itertools::{izip, Itertools};
use std::{
    cmp::{max, min, Ordering},
    collections::VecDeque,
    isize,
    ops::Mul,
    vec,
};

use super::symbolic::{Node, NodeAttrs};

pub enum MovementOps {
    Reshape,
    Permute,
    Expand,
    Pad,
    Shrink,
    Stride,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeOrInt {
    Node(Node),
    Int(isize),
}

impl NodeOrInt {
    #[inline]
    pub fn is_var(&self) -> bool {
        matches!(self, NodeOrInt::Node(Node::Var { .. }))
    }

    #[inline]
    pub fn is_int(&self) -> bool {
        matches!(self, NodeOrInt::Int(_))
    }

    pub fn as_var_mut(&mut self) -> (&mut String, &mut Option<isize>, &mut NodeAttrs) {
        match self {
            NodeOrInt::Node(Node::Var {
                ref mut expr,
                ref mut val,
                ref mut attrs,
            }) => (expr, val, attrs),
            _ => panic!("NodeOrInt is not a var"),
        }
    }

    pub fn as_int(&self) -> isize {
        match self {
            NodeOrInt::Int(i) => *i,
            _ => panic!("NodeOrInt is not an int"),
        }
    }
}

impl From<isize> for NodeOrInt {
    fn from(i: isize) -> Self {
        NodeOrInt::Int(i)
    }
}

pub fn to_shape_strides(shape: &[NodeOrInt], strides: &[isize]) -> Vec<(isize, isize)> {
    debug_assert!(shape.len() == strides.len());
    let mut result: Vec<(isize, isize)> = Vec::new();
    if !shape.is_empty() {
        result.push((shape[0].as_int(), strides[0]));
    }

    for i in 1..shape.len() {
        if (strides[i] != 0 && result.last().unwrap().1 == shape[i].as_int() * strides[i])
            || result.last().unwrap().0 == 1
            || (strides[i] == 0 && result.last().unwrap().1 == 0)
        {
            result.last_mut().unwrap().0 *= shape[i].as_int();
            result.last_mut().unwrap().1 = strides[i];
        } else {
            result.push((shape[i].as_int(), strides[i]));
        }
    }

    result
}

pub fn is_contiguous(shape: &[NodeOrInt], strides: &[isize]) -> bool {
    debug_assert!(shape.len() == strides.len());

    izip!(shape, strides, &strides_for_shape(shape))
        .all(|(s, s1, s2)| *s1 == *s2 || s.as_int() == 1)
}

pub fn filter_strides(shape: &[NodeOrInt], strides: &[isize]) -> Vec<isize> {
    debug_assert!(shape.len() == strides.len());

    izip!(strides, shape)
        .map(|(s, shp)| if shp.as_int() != 1 { *s } else { 0 })
        .collect()
}

// class ViewInternal(NamedTuple):
//   shape:Tuple[int, ...]
//   strides:Tuple[int, ...]
//   offset:int
//   mask:Optional[Tuple[Tuple[int, int]]]
//   contiguous:bool
//   shape_strides:Tuple[Tuple[int, int], ...]

#[derive(Debug, Clone)]
pub struct View {
    pub shape: Vec<NodeOrInt>,
    pub strides: Vec<isize>,
    pub offset: isize,
    pub mask: Option<Vec<(isize, isize)>>,
    pub contiguous: bool,
    pub shape_strides: Vec<(isize, isize)>,
}

impl View {
    pub fn new(
        shape: &[NodeOrInt],
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
            shape: shape.iter().map(|x| NodeOrInt::Int(x.as_int())).collect(),
            strides: filtered_strides,
            offset,
            mask,
            contiguous,
            shape_strides,
        }
    }

    pub fn expr_node_mask(&self, idx: &Node, valid: Option<&Node>) -> Node {
        let mut expr = match valid {
            Some(v) => vec![v.clone()],
            None => Vec::new(),
        };

        if let Some(mask) = &self.mask {
            let mut acc = 1;
            for (ns, (x, y)) in self.shape.iter().zip(mask.iter()).rev() {
                let base = (idx.clone().floordiv(acc, None)).modulus(ns.as_int());
                expr.push(base.clone().ge(*x));
                expr.push(base.lt(*y));
                acc *= ns.as_int();
            }
        }

        Node::new_ands(expr.as_ref())
    }

    pub fn expr_node(&self, idx: Option<&Node>) -> Node {
        let idx = match idx {
            None => Node::new_var(
                "idx",
                0,
                self.shape.iter().map(|x| x.as_int()).product::<isize>() - 1,
            ),
            Some(idx) => idx.clone(),
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

        for (idx, sh, st) in izip!(idxs.iter(), self.shape.iter(), self.strides.iter()) {
            if sh.as_int() != 1 && *st != 0 {
                result.push(idx.clone().mul(*st));
            }
        }

        Node::new_sum(result.as_ref())
    }
}

pub fn idxs_to_idx(shape: &[NodeOrInt], idxs: &[Node]) -> Node {
    debug_assert!(idxs.len() == shape.len());

    let mut acc: isize = 1;
    let mut result = Vec::new();
    for (idx, d) in idxs.iter().zip(shape.iter()).rev() {
        result.push(idx.clone().mul(acc));
        acc *= d.as_int();
    }

    Node::new_sum(result.as_ref())
}

pub fn strides_for_shape(shape: &[NodeOrInt]) -> Vec<isize> {
    let mut strides: Vec<isize> = Vec::new();
    if !shape.is_empty() {
        strides.push(1);
    }

    for d in shape.iter().rev().skip(1) {
        strides.insert(0, d.as_int() * strides[0]);
    }

    filter_strides(shape, &strides)
}

pub fn merge_views(vm2: &View, vm1: &View) -> Option<View> {
    if vm2.mask.is_some() {
        return None;
    }

    let mst = ShapeTracker::with_views(&[vm2.clone(), vm1.clone()]);
    let strides = mst.real_strides(false);
    if strides.iter().any(|x| x.is_none()) {
        return None;
    }

    let strides = strides
        .iter()
        .map(|x| match x {
            Some(Node::Num { attrs }) => attrs.b,
            _ => panic!("Unexpected stride node type"),
        })
        .collect::<Vec<_>>();

    Some(View::new(
        &vm1.shape,
        Some(&strides),
        mst.real_offset(),
        vm1.mask.as_deref(),
    ))
}

fn _reshape(view: &View, new_shape: &[NodeOrInt]) -> (View, bool) {
    let filtered_shape: Vec<isize> = view
        .shape
        .iter()
        .filter(|x| x.as_int() != 1)
        .map(|x| x.as_int())
        .collect();
    let filtered_newshape: Vec<isize> = new_shape
        .iter()
        .filter(|x| x.as_int() != 1)
        .map(|x| x.as_int())
        .collect();

    if filtered_shape == filtered_newshape {
        let mut new_strides: VecDeque<isize> = izip!(view.shape.iter(), view.strides.iter())
            .filter(|(x, _)| x.as_int() != 1)
            .map(|(_, y)| *y)
            .collect();

        let new_strides_tuple: Vec<isize> = new_shape
            .iter()
            .map(|x| {
                if x.as_int() == 1 {
                    0
                } else {
                    new_strides.pop_front().unwrap()
                }
            })
            .collect();

        let mut new_mask_tuple = None;
        if let Some(ref mask1) = view.mask {
            for (x, y) in izip!(view.shape.iter(), mask1.iter()) {
                if x.as_int() == 1 && *y != (0, 1) {
                    new_mask_tuple = Some(vec![(0, 0); new_shape.len()]);
                    break;
                } else {
                    let mut new_mask: Vec<(isize, isize)> =
                        izip!(view.shape.iter(), view.mask.as_ref().unwrap().iter())
                            .filter(|(x, _)| x.as_int() != 1)
                            .map(|(_, y)| *y)
                            .rev()
                            .collect();

                    new_mask_tuple = Some(
                        new_shape
                            .iter()
                            .map(|x| {
                                if x.as_int() == 1 {
                                    (0, 1)
                                } else {
                                    new_mask.pop().unwrap()
                                }
                            })
                            .collect(),
                    );
                }
            }
        }

        let new_mask_tuple: Option<&[(isize, isize)]> = match new_mask_tuple {
            Some(ref x) => Some(x.as_slice()),
            None => None,
        };

        return (
            View::new(
                new_shape,
                Some(new_strides_tuple.as_slice()),
                view.offset,
                new_mask_tuple,
            ),
            false,
        );
    }

    let new_view = View::new(new_shape, None, 0, None);
    if new_view.contiguous {
        return (new_view, false);
    }

    let merged_view = merge_views(view, &new_view);
    if let Some(view) = merged_view {
        return (view, false);
    }

    log::debug!(
        "WARNING: creating new view with reshape {:?} -> {:?}",
        new_view,
        new_shape
    );

    (new_view, true)
}

pub struct PadArgs {
    pub zvarg: Vec<(isize, isize)>,
    pub mask: Vec<(isize, isize)>,
}

pub fn get_pad_args(shape: &[NodeOrInt], arg: &[(isize, isize)]) -> PadArgs {
    let mut zvarg: Vec<(isize, isize)> = Vec::with_capacity(shape.len());
    let mut mask: Vec<(isize, isize)> = Vec::with_capacity(shape.len());

    for (s, (b, e)) in izip!(shape.iter(), arg.iter()) {
        zvarg.push((-b, s.as_int() + e));
        mask.push((*b, s.as_int() + b));
    }

    PadArgs { zvarg, mask }
}

pub fn get_unsafe_resize_offset(strides: &[isize], arg: &[(isize, isize)]) -> isize {
    izip!(strides, arg).map(|(s, x)| s * x.0).sum()
}

#[derive(Debug, Clone)]
pub struct ShapeTracker {
    pub views: Vec<View>,
}

impl ShapeTracker {
    pub fn with_shape(shape: &[NodeOrInt]) -> Self {
        ShapeTracker {
            views: vec![View::new(shape, None, 0, None)],
        }
    }

    pub fn with_views(views: &[View]) -> Self {
        ShapeTracker {
            views: views.to_vec(),
        }
    }

    pub fn contiguous(&self) -> bool {
        self.views.len() == 1 && self.views[0].contiguous
    }

    pub fn shape(&self) -> Vec<NodeOrInt> {
        self.views.last().unwrap().shape.clone()
    }

    pub fn size(&self) -> isize {
        izip!(
            self.views.last().unwrap().shape.iter(),
            self.views.last().unwrap().strides.iter()
        )
        .filter(|(_, st)| **st != 0)
        .map(|(s, _)| s.as_int())
        .product()
    }

    pub fn real_offset(&self) -> isize {
        let (real_offset, mask) = self.expr_node(&Node::new_var("zero", 0, 0));
        match real_offset {
            Node::Num { attrs } => attrs.b,
            _ => panic!(
                "how is the offset not a number? {:?} {:?}",
                real_offset, mask
            ),
        }
    }

    pub fn real_strides(&self, ignore_valid: bool) -> Vec<Option<Node>> {
        if self.views.len() == 1 && self.views.last().unwrap().mask.is_none() {
            return self
                .views
                .last()
                .unwrap()
                .strides
                .iter()
                .map(|x| Some(Node::new_num(*x)))
                .collect();
        }

        let idxs = (0..self.shape().len())
            .map(|i| Node::new_var(&format!("idx{}", i), 0, self.shape()[i].as_int() - 1))
            .collect::<Vec<Node>>();

        let (idx, valid) = self.expr_idxs(Some(&idxs));
        let mut ret: Vec<Option<Node>> = vec![None; self.views.last().unwrap().shape.len()];

        let tmp = vec![idx.clone()];
        let nodes = match &idx {
            Node::Sum { nodes, .. } => nodes,
            _ => &tmp,
        };

        for this_dim in nodes {
            match this_dim {
                Node::Mult { a, attrs } => {
                    if let Node::Var { .. } = **a {
                        ret[idxs.iter().position(|x| *x == **a).unwrap()] =
                            Some(Node::new_num(attrs.b));
                    }
                }
                Node::Var { .. } => {
                    ret[idxs.iter().position(|x| *x == *this_dim).unwrap()] =
                        Some(Node::new_num(1));
                }
                _ => (),
            }
        }

        let (idx_vars, valid_vars) = (idx.vars(), valid.vars());
        for (i, tidx) in idxs.iter().enumerate() {
            if valid_vars.iter().any(|x| *x == tidx) && !ignore_valid {
                ret[i] = None;
            } else if !idx_vars.iter().any(|x| *x == tidx) {
                ret[i] = Some(Node::new_num(0));
            }
        }

        ret
    }

    pub fn unit_stride_axes(&self, ignore_valid: bool) -> Vec<isize> {
        self.real_strides(ignore_valid)
            .iter()
            .enumerate()
            .filter(|(_, st)| **st == Some(Node::new_num(1)))
            .map(|(i, _)| i as isize)
            .collect()
    }

    fn _expr_idx(&self, idx: &Node, valid: &Node) -> (Node, Node) {
        let mut idx = idx.clone();
        let mut valid = valid.clone();
        for v in self.views.iter().rev().skip(1) {
            valid = v.expr_node_mask(&idx, Some(&valid));
            idx = v.expr_node(Some(&idx));
        }
        (idx, valid)
    }

    //   def simplify(self):
    //     if len(self.views) >= 2:
    //       new_view = merge_views(self.views[-2], self.views[-1])
    //       if new_view:
    //         if DEBUG >= 4: print(f"st simplify : {self.views[-2]} + {self.views[-1]} = {new_view}")
    //         self.views = self.views[:-2] + [new_view]
    //         self.simplify()
    pub fn simplify(&mut self) {
        if self.views.len() >= 2 {
            let new_view = merge_views(
                self.views.get(self.views.len() - 2).unwrap(),
                self.views.last().unwrap(),
            );

            if let Some(new_view) = new_view {
                log::debug!(
                    "st simplify : {:?} + {:?} = {:?}",
                    self.views.get(self.views.len() - 2).unwrap(),
                    self.views.last().unwrap(),
                    new_view
                );

                self.views = self.views[..self.views.len() - 2].to_vec();
                self.views.push(new_view);
                self.simplify();
            }
        }
    }

    pub fn expr_idxs(&self, idxs: Option<&[Node]>) -> (Node, Node) {
        let idxs = match idxs {
            Some(idxs) => idxs.to_vec(),
            None => self
                .shape()
                .iter()
                .enumerate()
                .map(|(i, s)| Node::new_var(&format!("idx{}", i), 0, s.as_int() - 1))
                .collect(),
        };
        let idx = self.views.last().unwrap().expr_idxs(&idxs);
        let valid = self
            .views
            .last()
            .unwrap()
            .expr_node_mask(&idxs_to_idx(&self.views.last().unwrap().shape, &idxs), None);
        self._expr_idx(&idx, &valid)
    }

    pub fn expr_node(&self, idx: &Node) -> (Node, Node) {
        self._expr_idx(
            &self.views.last().unwrap().expr_node(Some(idx)),
            &self.views.last().unwrap().expr_node_mask(idx, None),
        )
    }

    pub fn needs_valid(&self) -> bool {
        self.views.iter().any(|v| v.mask.is_some())
    }

    fn unsafe_resize(
        &mut self,
        arg: &[(isize, isize)],
        mask: Option<&[(isize, isize)]>,
    ) -> &mut Self {
        let offset = get_unsafe_resize_offset(&self.views.last().unwrap().strides, arg);
        let mut new_mask = None;
        if let Some(m1) = &self.views.last().unwrap().mask {
            // move the old mask
            let nmask = m1
                .iter()
                .zip(arg)
                .map(|((mx, my), (ax, ay))| (max(*mx - *ax, 0), min(*my - *ax, *ay - *ax)))
                .collect::<Vec<_>>();
            // merge the masks if we have two
            new_mask = if let Some(m2) = mask {
                Some(
                    izip!(m2.iter(), nmask.as_slice().iter())
                        .map(|((mx1, my1), (mx2, my2))| (max(*mx1, *mx2), min(*my1, *my2)))
                        .collect::<Vec<_>>(),
                )
            } else {
                Some(nmask)
            };
        }

        let idx = self.views.len() - 1;
        self.views[idx] = View::new(
            arg.iter()
                .map(|(x, y)| NodeOrInt::Int(*y - *x))
                .collect::<Vec<_>>()
                .as_slice(),
            Some(self.views[idx].strides.as_slice()),
            self.views[idx].offset + offset,
            new_mask.as_deref(),
        );
        self
    }

    pub fn pad(&mut self, arg: &[(isize, isize)]) -> &mut Self {
        debug_assert!(
            arg.iter().all(|(b, e)| *b >= 0 && *e >= 0) && arg.len() == self.shape().len()
        );

        if arg.iter().any(|(b, e)| *b != 0 || *e != 0) {
            let PadArgs { zvarg, mask } = get_pad_args(self.shape().as_slice(), arg);
            self.unsafe_resize(&zvarg, Some(&mask));
        }
        self
    }

    pub fn shrink(&mut self, arg: &[(isize, isize)]) -> &mut Self {
        debug_assert!(arg
            .iter()
            .all(|(b, e)| *b >= 0 && *e <= self.shape().len() as isize));
        self.unsafe_resize(arg, None);
        self
    }

    pub fn expand(&mut self, new_shape: &[NodeOrInt]) -> &mut Self {
        debug_assert!(new_shape.len() == self.views.last().unwrap().shape.len());
        let mask = self.views.last().unwrap().mask.as_ref().map(|mask| {
            mask.iter()
                .zip(self.shape().iter())
                .zip(new_shape.iter())
                .map(|((m, s), ns)| {
                    if m != &(0, 1) {
                        (0, 0)
                    } else if s.as_int() != ns.as_int() {
                        (0, ns.as_int())
                    } else {
                        *m
                    }
                })
                .collect::<Vec<_>>()
        });

        let idx = self.views.len() - 1;
        self.views[idx] = View::new(
            new_shape,
            Some(self.views[idx].strides.as_slice()),
            self.views[idx].offset,
            mask.as_deref(),
        );
        self
    }

    //   def reshape(self, new_shape: Tuple[Union[Node,int], ...]):
    //     # reshape into symbolic shape, update the variable value
    //     if all(isinstance(s, int) for s in self.shape) and len(new_vars:=list(s for s in new_shape if isinstance(s, Variable))) > 0:
    //       assert len(new_vars) == 1, "only one variable is supported in a shape"
    //       new_var, new_val = new_vars[0], prod(self.shape) // prod(s for s in new_shape if isinstance(s, int))
    //       if new_var.val is None:
    //         assert new_var.min <= new_val <= new_var.max, f"variable value {new_val} out of range [{new_var.min}, {new_var.max}]"
    //         new_var.val = new_val
    //       else: assert new_var.val == new_val, f"value conflicts, was {new_var.val}, set to {new_val}"
    //     if self.views[-1].shape == new_shape: return self
    //     assert all(is_sym_int(x) and x > 0 for x in new_shape), f"shape must be symbolic ints and can't contain 0 or negative numbers {new_shape}"
    //     # only check size for int shapes. we don't check symbolic here as long as the reshape itself can be done
    //     if all(isinstance(s, int) for s in self.shape) and all(isinstance(s, int) for s in new_shape):
    //       assert prod(self.shape) == prod(new_shape), f"can't reshape {self.shape} -> {new_shape}" # type: ignore  # mypy cannot resolve, all ints here
    //     new_view, extra = _reshape(self.views[-1], new_shape)
    //     if extra: self.views.append(new_view)
    //     else: self.views[-1] = new_view
    //     return self
    pub fn reshape(&mut self, new_shape: &mut [NodeOrInt]) -> &mut Self {
        let new_shape_prod = new_shape
            .iter()
            .filter(|x| x.is_int())
            .map(|x| x.as_int())
            .product::<isize>();
        let mut new_vars = new_shape
            .iter_mut()
            .filter(|x| x.is_var())
            .collect::<Vec<_>>();
        if !new_vars.is_empty() {
            debug_assert!(
                new_vars.len() == 1,
                "only one variable is supported in a shape"
            );
            let mut new_var = new_vars[0].as_var_mut();
            let new_val =
                self.shape().iter().map(|x| x.as_int()).product::<isize>() / new_shape_prod;
            if new_var.1.is_none() {
                debug_assert!(
                    new_var.2.min <= new_val && new_val <= new_var.2.max,
                    "variable value {} out of range [{}, {}]",
                    new_val,
                    new_var.2.min,
                    new_var.2.max
                );
                new_var.1 = &mut Some(new_val);
            } else {
                debug_assert!(
                    new_var.1 == &mut Some(new_val),
                    "value conflicts, was {}, set to {}",
                    new_var.1.unwrap(),
                    new_val
                );
            }
        }

        if self.views.last().unwrap().shape == new_shape {
            return self;
        }

        let (new_view, extra) = _reshape(self.views.last().unwrap(), new_shape);
        if extra {
            self.views.push(new_view);
        } else {
            let idx = self.views.len() - 1;
            self.views[idx] = new_view;
        }

        self
    }

    pub fn permute(&mut self, axis: &[NodeOrInt]) -> &mut Self {
        debug_assert!(
            axis.iter()
                .all(|x| x.as_int() >= 0 && x.as_int() < self.shape().len() as isize),
            "invalid permute {:?} for {:?}",
            axis,
            self.shape()
        );
        debug_assert!(
            axis.len() == self.shape().len(),
            "can't permute {:?} with {:?}",
            self.shape(),
            axis
        );

        let idx = self.views.len() - 1;
        let shape = axis
            .iter()
            .map(|ref a| &self.views[idx].shape[a.as_int() as usize])
            .map(|x| x.clone())
            .collect_vec();
        let strides: Vec<isize> = axis
            .iter()
            .map(|ref a| self.views[idx].strides[a.as_int() as usize])
            .collect();
        let offset = self.views[idx].offset;
        let mask = self.views[idx].mask.as_ref().map(|mask| {
            axis.iter()
                .map(|ref a| mask[a.as_int() as usize].clone())
                .collect::<Vec<(isize, isize)>>()
        });

        self.views[idx] = View::new(&shape, Some(&strides), offset, mask.as_deref());
        self
    }

    pub fn stride(&mut self, mul: &[NodeOrInt]) -> &mut Self {
        let strides = izip!(self.views.last().unwrap().strides.iter(), mul)
            .map(|(z, m)| z * m.as_int())
            .collect::<Vec<isize>>();

        let new_shape: Vec<NodeOrInt> = izip!(self.views.last().unwrap().shape.iter(), mul)
            .map(|(s, m)| NodeOrInt::Int((s.as_int() + (m.as_int().abs() - 1)) / m.as_int().abs()))
            .collect();

        let offset = izip!(
            self.views.last().unwrap().shape.iter(),
            self.views.last().unwrap().strides.iter(),
            mul
        )
        .filter(|(_, _, m)| m.as_int() < 0)
        .map(|(s, z, m)| (s.as_int() - 1) * z)
        .sum();

        let mask = if let Some(mask) = &self.views.last().unwrap().mask {
            Some(
                izip!(mask.iter(), self.views.last().unwrap().shape.iter(), mul)
                    .map(|((mx, my), s, m)| {
                        (
                            ((if m.as_int() > 0 {
                                *mx
                            } else {
                                s.as_int() - *my
                            }) + (m.as_int().abs() - 1))
                                / m.as_int().abs(),
                            ((if m.as_int() > 0 {
                                *my
                            } else {
                                s.as_int() - *mx
                            }) + (m.as_int().abs() - 1))
                                / m.as_int().abs(),
                        )
                    })
                    .collect::<Vec<(isize, isize)>>(),
            )
        } else {
            None
        };

        let idx = self.views.len();
        self.views[idx - 1] = View::new(&new_shape, Some(&strides), offset, mask.as_deref());
        self
    }

    pub fn movement_op(&mut self, op: MovementOps, arg: &mut [NodeOrInt]) -> &mut Self {
        match op {
            MovementOps::Reshape => self.reshape(arg),
            MovementOps::Expand => self.expand(arg),
            MovementOps::Permute => self.permute(arg),
            MovementOps::Stride => self.stride(arg),
            _ => panic!("invalid movement op"),
        }
    }

    pub fn movement_op1(&mut self, op: MovementOps, arg: &[(isize, isize)]) -> &mut Self {
        match op {
            MovementOps::Pad => self.pad(arg),
            MovementOps::Shrink => self.shrink(arg),
            _ => panic!("invalid movement op"),
        }
    }
}

pub fn get_contraction(old_shape: &[isize], new_shape: &[isize]) -> Option<Vec<Vec<usize>>> {
    let mut axis_groups: Vec<Vec<usize>> = vec![Vec::new(); new_shape.len()];
    let mut i = 0;
    let mut old_shape_i = 0;
    while old_shape_i < old_shape.len() {
        if new_shape[i] == 1 && old_shape[old_shape_i] != 1 {
            if i < new_shape.len() - 1 {
                i += 1;
            }
        } else {
            axis_groups[i].push(old_shape_i);
            let axis_group_size = axis_groups[i]
                .iter()
                .map(|x| old_shape[*x])
                .product::<isize>();

            match axis_group_size.cmp(&new_shape[i]) {
                Ordering::Equal => {
                    if i < new_shape.len() - 1 {
                        i += 1;
                    }
                }
                Ordering::Greater => return None,
                Ordering::Less => {}
            }

            old_shape_i += 1;
        }
    }

    Some(axis_groups)
}

#[cfg(test)]
mod tests {}
