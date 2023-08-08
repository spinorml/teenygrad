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
use std::{collections::VecDeque, ops::Mul, vec};

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

pub fn idxs_to_idx(shape: &[isize], idxs: &[Node]) -> Node {
    debug_assert!(idxs.len() == shape.len());

    let mut acc: isize = 1;
    let mut result = Vec::new();
    for (idx, d) in idxs.iter().zip(shape.iter()).rev() {
        result.push(idx.clone().mul(acc));
        acc *= d;
    }

    Node::new_sum(result.as_ref())
}

pub fn strides_for_shape(shape: &[isize]) -> Vec<isize> {
    let mut strides: Vec<isize> = Vec::new();
    if !shape.is_empty() {
        strides.push(1);
    }

    for d in shape.iter().rev().skip(1) {
        strides.insert(0, d * strides[0]);
    }

    filter_strides(shape, &strides)
}

pub fn merge_views(vm2: &View, vm1: &View) -> Option<View> {
    if vm2.mask.is_some() {
        return None;
    }

    // let mst = ShapeTracker::new(&vm1.shape, &[vm2, vm1]);
    // let strides = mst.real_strides();
    // if strides.contains(&None) {
    //     return None;
    // }

    // View::new(&vm1.shape, strides, ms.real_offset, vm1.mask)
    todo!()
}

// @functools.lru_cache(maxsize=None)
// def _reshape(view: View, new_shape:Tuple[int, ...]) -> Tuple[View, bool]:
//   shape, mask, strides, offset = view.shape, view.mask, view.strides, view.offset
//   # check if this is adding or removing 1s (only)
//   # NOTE: this is optional, but removes most calls to (expensive!) merge_views (with mask, not optional)
//   if [x for x in shape if x != 1] == [x for x in new_shape if x != 1]:
//     new_strides: List[int] = [y for x,y in zip(shape, strides) if x != 1]
//     new_strides_tuple: Tuple[int, ...] = tuple([0 if x == 1 else new_strides.pop(0) for x in new_shape])
//     new_mask_tuple = None
//     if mask:
//       for x,y in zip(shape, mask):
//         if x == 1 and y != (0, 1):
//           new_mask_tuple = ((0,0),) * len(new_shape)
//           break
//       else:
//         new_mask: List[Tuple[int, int]] = [y for x,y in zip(shape, mask) if x != 1]
//         new_mask_tuple = tuple([(0,1) if x == 1 else new_mask.pop(0) for x in new_shape])
//     return View(new_shape, new_strides_tuple, offset, new_mask_tuple), False

//   new_view = View(new_shape)
//   if view.contiguous: return new_view, False # NOTE: if it's contiguous it can't have an offset
//   if (merged_view := merge_views(view, new_view)) is not None: return merged_view, False
//   if DEBUG >= 4: print(f"WARNING: creating new view with reshape {view} -> {new_shape}")
//   return new_view, True

fn _reshape(view: &View, new_shape: &[isize]) -> (View, bool) {
    //   shape, mask, strides, offset = view.shape, view.mask, view.strides, view.offset
    let (shape, mask, strides, offset) = (view.shape, view.mask, view.strides, view.offset);

    //   if [x for x in shape if x != 1] == [x for x in new_shape if x != 1]:
    //     new_strides: List[int] = [y for x,y in zip(shape, strides) if x != 1]
    //     new_strides_tuple: Tuple[int, ...] = tuple([0 if x == 1 else new_strides.pop(0) for x in new_shape])
    //     new_mask_tuple = None
    //     if mask:
    //       for x,y in zip(shape, mask):
    //         if x == 1 and y != (0, 1):
    //           new_mask_tuple = ((0,0),) * len(new_shape)
    //           break
    //       else:
    //         new_mask: List[Tuple[int, int]] = [y for x,y in zip(shape, mask) if x != 1]
    //         new_mask_tuple = tuple([(0,1) if x == 1 else new_mask.pop(0) for x in new_shape])

    let lhs: Vec<isize> = shape.iter().filter(|x| **x != 1).map(|x| *x).collect();
    let rhs: Vec<isize> = new_shape.iter().filter(|x| **x != 1).map(|x| *x).collect();

    if lhs == rhs {
        let mut new_strides: VecDeque<isize> = shape
            .iter()
            .zip(strides.iter())
            .filter(|(x, _)| **x != 1)
            .map(|(_, y)| *y)
            .collect();
        let new_strides_tuple: Vec<isize> = new_shape
            .iter()
            .map(|x| {
                if *x == 1 {
                    0
                } else {
                    new_strides.pop_front().unwrap()
                }
            })
            .collect();
        let mut new_mask_tuple = None;
        if mask.is_some() {
            for (x, y) in izip!(shape.iter(), mask.unwrap().iter()) {
                if *x == 1 && *y != (0, 1) {
                    new_mask_tuple = Some(vec![(0, 0); new_shape.len()]);
                    break;
                }
            }
        }
        return (
            View::new(&new_shape, Some(&new_strides_tuple), offset, new_mask_tuple),
            false,
        );
    }
    todo!()
}

// @functools.lru_cache(maxsize=None)
// def get_pad_args(shape:Tuple[int,...], arg:Tuple[Tuple[int, int], ...]):
//   return tuple([(-b,s+e) for s,(b,e) in zip(shape, arg)]), tuple([(b,s+b) for s,(b,_) in zip(shape, arg)])

// @functools.lru_cache(maxsize=None)
// def get_unsafe_resize_offset(strides, arg):
//   return sum([s * x[0] for s, x in zip(strides,arg)])

// class ShapeTracker:
//   __slots__ = "views"
//   def __init__(self, shape:Union[ShapeTracker, Tuple[int, ...]], views:Optional[List[View]]=None):
//     self.views: List[View] = views if views is not None else ([*cast(ShapeTracker, shape).views] if shape.__class__ is ShapeTracker else [View(shape)])
//   def __repr__(self): return f"ShapeTracker(shape={self.views[-1].shape}, views={self.views})"
//   def copy(self) -> ShapeTracker: return ShapeTracker(self.views[-1].shape, [*self.views])

//   @property
//   def contiguous(self) -> bool: return len(self.views) == 1 and self.views[0].contiguous

//   @property
//   def shape(self) -> Tuple[int, ...]: return self.views[-1].shape

//   @property
//   def key(self) -> Tuple[View, ...]: return tuple(self.views)

//   # this is the real size (ish)
//   def size(self): return prod([s for s,st in zip(self.views[-1].shape, self.views[-1].strides) if st != 0])

//   # these are multiview strides, value is None if it's not a simple strided dimension
//   # TODO: this can be shared code between simplify and merge_views
//   def real_offset(self) -> int:
//     real_offset, mask = self.expr_node(Variable('zero', 0, 0))
//     assert real_offset.__class__ is NumNode, f"how is the offset not a number? {real_offset} {mask}"
//     return real_offset.b

//   # NOTE: if a stride is not always valid, it will be None
//   def real_strides(self, ignore_valid=False) -> Tuple[Optional[Union[Node, int]], ...]:
//     if len(self.views) == 1 and self.views[-1].mask is None: return self.views[-1].strides
//     idxs = [Variable(f"idx{i}", 0, s-1) for i,s in enumerate(self.shape)]
//     idx, valid = self.expr_idxs(idxs)
//     ret: List[Optional[Union[Node, int]]] = [None] * len(self.views[-1].shape)
//     for this_dim in (idx.nodes if isinstance(idx, SumNode) else [idx]):
//       if isinstance(this_dim, MulNode) and isinstance(this_dim.a, Variable):
//         ret[idxs.index(this_dim.a)] = this_dim.b
//       elif isinstance(this_dim, Variable):
//         ret[idxs.index(this_dim)] = 1
//     idx_vars, valid_vars = idx.vars(), valid.vars()
//     for i,tidx in enumerate(idxs):
//       if tidx in valid_vars and not ignore_valid: ret[i] = None
//       elif tidx not in idx_vars: ret[i] = 0
//     return tuple(ret)
//   def unit_stride_axes(self, ignore_valid=False) -> List[int]: return [i for i,st in enumerate(self.real_strides(ignore_valid)) if st == 1]

//   def _expr_idx(self, idx, valid):
//     for v in reversed(self.views[0:-1]):
//       valid = v.expr_node_mask(idx, valid)
//       idx = v.expr_node(idx)
//     return idx, valid

//   def simplify(self):
//     if len(self.views) >= 2:
//       new_view = merge_views(self.views[-2], self.views[-1])
//       if new_view:
//         if DEBUG >= 4: print(f"st simplify : {self.views[-2]} + {self.views[-1]} = {new_view}")
//         self.views = self.views[:-2] + [new_view]
//         self.simplify()

//   def expr_idxs(self, idxs=None):
//     if idxs is None: idxs = [Variable(f"idx{i}", 0, s-1) for i,s in enumerate(self.shape)]
//     idx = self.views[-1].expr_idxs(tuple(idxs))
//     valid = self.views[-1].expr_node_mask(idxs_to_idx(self.views[-1].shape, tuple(idxs)))
//     return self._expr_idx(idx, valid)

//   def expr_node(self, idx='idx'):
//     if idx.__class__ is str: idx = Variable(idx, 0, prod(self.shape)-1)
//     return self._expr_idx(self.views[-1].expr_node(idx), self.views[-1].expr_node_mask(idx))

//   def needs_valid(self) -> bool:
//     return any(v.mask is not None for v in self.views)

//   # *** under this line are the movement ops ***

//   def __unsafe_resize(self, arg: Tuple[Tuple[int, int], ...], mask=None):
//     offset = get_unsafe_resize_offset(self.views[-1].strides, arg)
//     if self.views[-1].mask:
//       # move the old mask
//       nmask = tuple([(max(mx-ax, 0), min(my-ax, ay-ax)) for (mx,my),(ax,ay) in zip(self.views[-1].mask, arg)])
//       # merge the masks if we have two
//       mask = tuple([(max(mx1, mx2), min(my1, my2)) for (mx1, my1), (mx2, my2) in zip(nmask, mask)]) if mask is not None else nmask
//     self.views[-1] = View(tuple([y-x for x,y in arg]), self.views[-1].strides, self.views[-1].offset+offset, mask)

//   def pad(self, arg: Tuple[Tuple[int, int], ...]):
//     assert all((b>=0 and e>=0) for b,e in arg) and len(arg) == len(self.shape)
//     if any(b or e for b, e in arg):
//       zvarg, mask = get_pad_args(self.shape, arg)
//       self.__unsafe_resize(zvarg, mask=mask)
//     return self

//   def shrink(self, arg: Tuple[Tuple[int, int], ...]):
//     assert all((b>=0 and e<=s) for s,(b,e) in zip(self.shape,arg)) and len(arg) == len(self.shape)
//     self.__unsafe_resize(arg)
//     return self

//   def expand(self, new_shape: Tuple[Union[Node,int], ...]) -> ShapeTracker:
//     assert len(new_shape) == len(self.views[-1].shape)
//     assert all(is_sym_int(x) and (s == x or (s == 1 and st == 0)) for s,x,st in zip(self.shape, new_shape, self.views[-1].strides)), f"can't expand {self.shape} into {new_shape}"
//     # NOTE: can the mask ever be (0,0)?
//     mask = tuple([(((0,0) if m != (0,1) else (0,ns)) if s != ns else m) for m,s,ns in zip(self.views[-1].mask, self.shape, new_shape)]) if self.views[-1].mask else None
//     self.views[-1] = View(new_shape, self.views[-1].strides, self.views[-1].offset, mask)
//     return self

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

//   def permute(self, axis: Tuple[int, ...]):
//     assert all(isinstance(x, int) and x >= 0 and x < len(self.shape) for x in axis), f"invalid permute {axis} for {self.shape}"
//     assert len(set(axis)) == len(axis) and len(axis) == len(self.shape), f"can't permute {self.shape} with {axis}"
//     self.views[-1] = View(tuple([self.views[-1].shape[a] for a in axis]), tuple([self.views[-1].strides[a] for a in axis]), self.views[-1].offset, tuple([self.views[-1].mask[a] for a in axis]) if self.views[-1].mask is not None else None)
//     return self

//   # except for the negative case, you can build this from the others. invertible in the negative case
//   def stride(self, mul: Tuple[int, ...]):
//     assert all(isinstance(x, int) and x != 0 for x in mul), f"invalid stride {mul} for {self.shape}"
//     strides = tuple([z*m for z,m in zip(self.views[-1].strides, mul)])
//     new_shape = tuple([(s+(abs(m)-1))//abs(m) for s,m in zip(self.views[-1].shape, mul)])
//     offset = sum([(s-1)*z for s,z,m in zip(self.views[-1].shape, self.views[-1].strides, mul) if m < 0])
//     mask = tuple([(((mx if m > 0 else s-my)+(abs(m)-1))//abs(m), ((my if m > 0 else s-mx)+(abs(m)-1))//abs(m)) for (mx,my),s,m in zip(self.views[-1].mask, self.views[-1].shape, mul)]) if self.views[-1].mask is not None else None
//     self.views[-1] = View(new_shape, strides, self.views[-1].offset + offset, mask)
//     return self

//   # *** entry point for external ***

//   def movement_op(self, op: MovementOps, arg:Union[Tuple[int, ...], Tuple[Tuple[int, int], ...]]) -> ShapeTracker:
//     assert isinstance(arg, tuple) and (len(arg) == len(self.shape) or op == MovementOps.RESHAPE), f"arg {arg} for {op} doesn't match dim of shape {self.shape}"
//     dispatch[op](self, arg)
//     return self

// dispatch: Dict[MovementOps, Callable] = {MovementOps.RESHAPE: ShapeTracker.reshape, MovementOps.EXPAND: ShapeTracker.expand, MovementOps.PAD: ShapeTracker.pad,
//                                          MovementOps.SHRINK: ShapeTracker.shrink, MovementOps.PERMUTE: ShapeTracker.permute, MovementOps.STRIDE: ShapeTracker.stride}

// # returns the axes to create new_shape if new_shape can be created by combining axis from old_shape
// def get_contraction(old_shape:Tuple[int, ...], new_shape:Tuple[int, ...]) -> Optional[List[List[int]]]:
//   # Pre-allocate all groups.
//   axis_groups: List[List[int]] = [[] for _ in range(len(new_shape))]
//   # Index for new_shape and axis_groups.
//   i: int = 0
//   old_shape_i: int = 0
//   while old_shape_i < len(old_shape):
//     # 1s exist in new_shape only will lead to empty axes group creations.
//     if new_shape[i] == 1 and old_shape[old_shape_i] != 1:
//       if i < len(new_shape) - 1: i += 1
//     else:
//       axis_groups[i].append(old_shape_i)
//       axis_group_size = prod([old_shape[x] for x in axis_groups[i]])
//       # Move to next axes group if total size of all dimensions match.
//       if axis_group_size == new_shape[i]:
//         if i < len(new_shape) - 1: i += 1
//       elif axis_group_size > new_shape[i]: return None
//       old_shape_i += 1
//   return axis_groups

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
