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

use std::{collections::HashSet, vec};

use crate::helpers::{gcd, partition};

pub trait Node {
    fn expr(&self) -> Option<&str> {
        None
    }

    fn a(&self) -> Option<&dyn Node> {
        None
    }

    fn b(&self) -> Option<&dyn Node> {
        None
    }

    fn min(&self) -> Option<isize> {
        None
    }

    fn max(&self) -> Option<isize> {
        None
    }

    fn vars(&self) -> Vec<&dyn Node> {
        let mut v: Vec<&dyn Node> = vec![];

        if let Some(a) = self.a() {
            v.extend(a.vars());
        }

        if let Some(b) = self.b() {
            v.extend(b.vars());
        }

        self.nodes().iter().for_each(|x| {
            v.extend(x.vars());
        });

        v
    }

    fn is_num(&self) -> bool {
        false
    }

    fn is_var(&self) -> bool {
        false
    }

    fn is_mul(&self) -> bool {
        false
    }

    fn is_sum(&self) -> bool {
        false
    }

    fn is_and(&self) -> bool {
        false
    }

    fn intval(&self) -> Option<isize> {
        None
    }

    fn as_bool(&self) -> bool {
        !(self.min() == self.max() && self.min().unwrap() == 0)
    }

    fn nodes(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String;

    fn clone(&self) -> Box<dyn Node>;

    fn key(&self) -> String {
        self.render(true, false)
    }

    fn eq(&self, other: &dyn Node) -> bool {
        self.key() == other.key()
    }

    fn get_bounds(&self) -> Option<(isize, isize)> {
        None
    }

    fn neg(&self) -> Box<dyn Node> {
        self.mul(num(-1).as_ref())
    }

    fn add(&self, other: &dyn Node) -> Box<dyn Node> {
        let tmp1 = self.clone();

        sum(&[tmp1.as_ref(), other])
    }

    fn sub(&self, other: &dyn Node) -> Box<dyn Node> {
        self.add(other.neg().as_ref())
    }

    fn mul(&self, b: &dyn Node) -> Box<dyn Node> {
        if b.is_num() {
            match b.intval().unwrap() {
                0 => return num(0),
                1 => return self.clone(),
                _ => {}
            }
        }

        if self.is_num() {
            if b.is_num() {
                return num(self.intval().unwrap() * b.intval().unwrap());
            }

            return b.mul(self.clone().as_ref());
        }

        create_node(MulNode::new(self.clone().as_ref(), b).as_ref())
    }

    fn floordiv(&self, b: &dyn Node, facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        if self.is_num() && b.is_num() {
            return num(self.intval().unwrap() / b.intval().unwrap());
        }

        node_floordiv(self.clone().as_ref(), b, facatoring_allowed)
    }

    fn modulus(&self, b: &dyn Node) -> Box<dyn Node> {
        if b.is_num() {
            let b_intval = b.intval().unwrap();
            match b_intval {
                1 => return num(0),
                _ => {
                    if self.is_num() {
                        let self_intval = self.intval().unwrap();
                        return num(self_intval % b_intval);
                    }
                }
            }
        }

        if self.key() == b.key() {
            return num(0);
        }

        create_node(ModNode::new(self.clone().as_ref(), b).as_ref())
    }

    fn le(&self, other: &dyn Node) -> Box<dyn Node> {
        self.lt(other.add(num(1).as_ref()).as_ref())
    }

    fn gt(&self, other: &dyn Node) -> Box<dyn Node> {
        self.neg().le(other.neg().as_ref())
    }

    fn ge(&self, other: &dyn Node) -> Box<dyn Node> {
        self.neg().lt(other.neg().add(num(1).as_ref()).as_ref())
    }

    fn lt(&self, b: &dyn Node) -> Box<dyn Node> {
        let mut result = self.clone();

        if result.is_sum() && b.is_num() {
            let nodes = result.nodes();
            let (muls, others) = partition(nodes.as_slice(), |x| {
                x.is_mul() && x.b().unwrap().intval().unwrap() > 0 && x.max().unwrap() >= 0
            });

            if !muls.is_empty() {
                let mut mul_gcd = muls.get(0).unwrap().b().unwrap().intval().unwrap();
                for x in muls.iter().skip(1) {
                    mul_gcd = gcd(mul_gcd, x.b().unwrap().intval().unwrap());
                }

                if b.modulus(num(mul_gcd).as_ref()).intval().unwrap() == 0 {
                    let all_others = sum(others.iter().map(|x| **x).collect::<Vec<_>>().as_slice());
                    if all_others.min().unwrap() >= 0 && all_others.max().unwrap() < mul_gcd {
                        result = sum(muls.iter().map(|x| **x).collect::<Vec<_>>().as_slice());
                    }
                }
            }
        }

        create_node(LtNode::new(result.as_ref(), b).as_ref())
    }

    fn flat_components(&self) -> Vec<Box<dyn Node>> {
        let mut components: Vec<Box<dyn Node>> = vec![];

        if self.is_sum() {
            self.nodes().iter().for_each(|x| {
                if x.is_sum() {
                    components.extend(x.flat_components())
                } else {
                    components.push((*x).clone());
                }
            });
        } else {
            components.push(self.clone());
        }

        components
    }
}

fn node_floordiv(a: &dyn Node, b: &dyn Node, _facatoring_allowed: Option<bool>) -> Box<dyn Node> {
    if b.is_num() {
        let b_intval = b.intval().unwrap();
        if b_intval == 1 {
            return a.clone();
        }

        if a.is_num() {
            let a_intval = a.intval().unwrap();
            return num(a_intval / b_intval);
        }
    }

    create_node(DivNode::new(a.clone().as_ref(), b).as_ref())
}

/*------------------------------------------------------*
| Utility functions
*-------------------------------------------------------*/

pub fn num(value: isize) -> Box<dyn Node> {
    NumNode::new(value)
}

pub fn var(expr: &str, min: isize, max: isize) -> Box<dyn Node> {
    create_node(Var::new(expr, min, max).as_ref())
}

pub fn factorize(_nodes: &[Box<dyn Node>]) -> Box<dyn Node> {
    todo!("factorize")
}

pub fn sum(nodes: &[&dyn Node]) -> Box<dyn Node> {
    let nodes = nodes
        .iter()
        .filter(|x| x.min().unwrap() != 0 || x.max().unwrap() != 0)
        .collect::<Vec<_>>();
    match nodes.len() {
        0 => return num(0),
        1 => return (*nodes[0]).clone(),
        _ => {}
    }

    let mut new_nodes: Vec<Box<dyn Node>> = vec![];
    let mut num_node_sum = 0;

    for node in nodes.iter().flat_map(|x| x.flat_components()) {
        if node.is_num() {
            num_node_sum += node.intval().unwrap();
        } else {
            new_nodes.push(node);
        }
    }

    if !new_nodes.is_empty() {
        let y = new_nodes
            .iter()
            .map(|x| {
                if x.is_mul() {
                    x.a().unwrap().render(true, false)
                } else {
                    x.render(true, false)
                }
            })
            .collect::<HashSet<_>>();
        if y.len() < new_nodes.len() {
            new_nodes = vec![factorize(&new_nodes)];
        }
    }

    if num_node_sum != 0 {
        new_nodes.push(num(num_node_sum));
    }

    match new_nodes.len() {
        0 => num(0),
        1 => new_nodes[0].clone(),
        _ => SumNode::new(&new_nodes),
    }
}

pub fn ands(nodes: &[&dyn Node]) -> Box<dyn Node> {
    //     if not nodes: return NumNode(1)
    // if len(nodes) == 1: return nodes[0]
    // if any(not x for x in nodes): return NumNode(0)

    // # filter 1s
    // nodes = [x for x in nodes if x.min != x.max]
    // return create_rednode(AndNode, nodes) if len(nodes) > 1 else (nodes[0] if len(nodes) == 1 else NumNode(1))

    match nodes.len() {
        0 => return num(1),
        1 => return (*nodes[0]).clone(),
        _ => (),
    }

    if nodes.iter().any(|x| !x.as_bool()) {
        return num(0);
    }

    let nodes = nodes
        .iter()
        .filter(|x| x.min() != x.max())
        .map(|x| (*x).clone())
        .collect::<Vec<_>>();

    match nodes.len() {
        1 => nodes[0].clone(),
        _ => AndNode::new(&nodes),
    }
}

fn create_node(node: &dyn Node) -> Box<dyn Node> {
    if node.min() == node.max() {
        num(node.min().unwrap())
    } else {
        node.clone()
    }
}

/*------------------------------------------------------*
| Variable
*-------------------------------------------------------*/
pub struct Var {
    expr: String,
    min: isize,
    max: isize,
}

impl Var {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(expr: &str, min: isize, max: isize) -> Box<dyn Node> {
        debug_assert!(min >= 0 && min <= max);

        if min == max {
            return num(min);
        }

        Box::new(Var {
            expr: expr.to_string(),
            min,
            max,
        })
    }
}

impl Node for Var {
    fn expr(&self) -> Option<&str> {
        Some(&self.expr)
    }

    fn min(&self) -> Option<isize> {
        Some(self.min)
    }

    fn max(&self) -> Option<isize> {
        Some(self.max)
    }

    fn is_var(&self) -> bool {
        true
    }

    fn render(&self, debug: bool, _strip_parens: bool) -> String {
        if debug {
            format!("{}[{}, {}]", self.expr, self.min, self.max)
        } else {
            self.expr.clone()
        }
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(Var {
            expr: self.expr.clone(),
            min: self.min,
            max: self.max,
        })
    }

    fn vars(&self) -> Vec<&dyn Node> {
        vec![self]
    }
}

/*------------------------------------------------------*
| Num
*-------------------------------------------------------*/
pub struct NumNode {
    value: isize,
}

impl NumNode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(value: isize) -> Box<dyn Node> {
        Box::new(NumNode { value })
    }
}

impl Node for NumNode {
    fn min(&self) -> Option<isize> {
        Some(self.value)
    }

    fn max(&self) -> Option<isize> {
        Some(self.value)
    }

    fn b(&self) -> Option<&dyn Node> {
        Some(self)
    }

    fn intval(&self) -> Option<isize> {
        Some(self.value)
    }

    fn is_num(&self) -> bool {
        true
    }

    fn render(&self, _debug: bool, _strip_parens: bool) -> String {
        self.value.to_string()
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(NumNode { value: self.value })
    }

    fn vars(&self) -> Vec<&dyn Node> {
        vec![self]
    }
}

/*------------------------------------------------------*
| LtNode
*-------------------------------------------------------*/

pub struct LtNode {
    a: Box<dyn Node>,
    b: Box<dyn Node>,
    min: isize,
    max: isize,
}

impl LtNode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(a: &dyn Node, b: &dyn Node) -> Box<dyn Node> {
        let mut node = LtNode {
            a: a.clone(),
            b: b.clone(),
            min: 0,
            max: 0,
        };

        let (min, max) = node.get_bounds().unwrap();
        node.min = min;
        node.max = max;

        Box::new(node)
    }
}

impl Node for LtNode {
    fn a(&self) -> Option<&dyn Node> {
        Some(self.a.as_ref())
    }

    fn b(&self) -> Option<&dyn Node> {
        Some(self.b.as_ref())
    }

    fn min(&self) -> Option<isize> {
        Some(self.min)
    }

    fn max(&self) -> Option<isize> {
        Some(self.max)
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };

        format!(
            "{}{}<{}{}",
            lparen,
            self.a.render(debug, strip_parens),
            self.b.render(debug, strip_parens),
            rparen
        )
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(LtNode {
            a: self.a.clone(),
            b: self.b.clone(),
            min: self.min,
            max: self.max,
        })
    }

    fn get_bounds(&self) -> Option<(isize, isize)> {
        if self.b.is_num() {
            let (a_min, a_max) = (self.a.min().unwrap(), self.a.max().unwrap());
            let b = self.b.intval().unwrap();

            let x: isize = if a_max < b { 1 } else { 0 };
            let y: isize = if a_min < b { 1 } else { 0 };
            return Some((x, y));
        }

        if self.a.max() < self.b.min() {
            Some((1, 1))
        } else if self.a.min() > self.b.max() {
            Some((0, 0))
        } else {
            Some((0, 1))
        }
    }

    fn floordiv(&self, b: &dyn Node, _: Option<bool>) -> Box<dyn Node> {
        let x = self.a.floordiv(b, None);
        let y = self.b.floordiv(b, None);
        x.lt(y.as_ref())
    }

    fn vars(&self) -> Vec<&dyn Node> {
        let mut v: Vec<&dyn Node> = vec![];
        v.extend(self.a().unwrap().vars());
        v.extend(self.b().unwrap().vars());
        v
    }
}

/*------------------------------------------------------*
| MulNode
*-------------------------------------------------------*/

pub struct MulNode {
    a: Box<dyn Node>,
    b: Box<dyn Node>,
    min: isize,
    max: isize,
}

impl MulNode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(a: &dyn Node, b: &dyn Node) -> Box<dyn Node> {
        let mut node = MulNode {
            a: a.clone(),
            b: b.clone(),
            min: 0,
            max: 0,
        };

        let (min, max) = node.get_bounds().unwrap();
        node.min = min;
        node.max = max;

        Box::new(node)
    }
}

impl Node for MulNode {
    fn a(&self) -> Option<&dyn Node> {
        Some(self.a.as_ref())
    }

    fn b(&self) -> Option<&dyn Node> {
        Some(self.b.as_ref())
    }

    fn min(&self) -> Option<isize> {
        Some(self.min)
    }

    fn max(&self) -> Option<isize> {
        Some(self.max)
    }

    fn is_mul(&self) -> bool {
        true
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };
        let a = self.a();
        let b = self.b();

        format!(
            "{}{}*{}{}",
            lparen,
            a.unwrap().render(debug, strip_parens),
            b.unwrap().render(debug, strip_parens),
            rparen
        )
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(MulNode {
            a: self.a.clone(),
            b: self.b.clone(),
            min: self.min,
            max: self.max,
        })
    }

    fn mul(&self, b: &dyn Node) -> Box<dyn Node> {
        self.a.mul(self.b.mul(b).as_ref())
    }

    fn floordiv(&self, b: &dyn Node, facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        let x = self.b.modulus(b);
        if x.is_num() && x.intval().unwrap() == 0 {
            return self.a.mul(self.b.floordiv(b, None).as_ref());
        }

        let x = b.modulus(self.b.as_ref());
        if x.is_num() && x.intval().unwrap() == 0 && self.b.is_num() && self.b.intval().unwrap() > 0
        {
            return self
                .a
                .floordiv(b.floordiv(self.b.as_ref(), None).as_ref(), None);
        }

        node_floordiv(self.clone().as_ref(), b, facatoring_allowed) // if self.b % b == 0: return self.a*(self.b//b)
    }

    fn modulus(&self, _other: &dyn Node) -> Box<dyn Node> {
        todo!("MulNode::modulus")
    }

    fn get_bounds(&self) -> Option<(isize, isize)> {
        let b = self.b.intval().unwrap();

        if b >= 0 {
            Some((self.a.min().unwrap() * b, self.a.max().unwrap() * b))
        } else {
            Some((self.a.max().unwrap() * b, self.a.min().unwrap() * b))
        }
    }
}

/*------------------------------------------------------*
| DivNode
*-------------------------------------------------------*/

pub struct DivNode {
    a: Box<dyn Node>,
    b: Box<dyn Node>,
    min: isize,
    max: isize,
}

impl DivNode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(a: &dyn Node, b: &dyn Node) -> Box<dyn Node> {
        let mut node = DivNode {
            a: a.clone(),
            b: b.clone(),
            min: 0,
            max: 0,
        };

        let (min, max) = node.get_bounds().unwrap();
        node.min = min;
        node.max = max;

        Box::new(node)
    }
}

impl Node for DivNode {
    fn min(&self) -> Option<isize> {
        Some(self.min)
    }

    fn max(&self) -> Option<isize> {
        Some(self.max)
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };

        format!(
            "{}{}//{}{}",
            lparen,
            self.a.render(debug, strip_parens),
            self.b.render(debug, strip_parens),
            rparen
        )
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(DivNode {
            a: self.a.clone(),
            b: self.b.clone(),
            min: self.min,
            max: self.max,
        })
    }

    fn get_bounds(&self) -> Option<(isize, isize)> {
        debug_assert!(self.a.min().unwrap() >= 0);

        let b = self.b.intval().unwrap();
        Some((self.a.min().unwrap() / b, self.a.max().unwrap() / b))
    }

    fn floordiv(&self, b: &dyn Node, _: Option<bool>) -> Box<dyn Node> {
        self.a.floordiv(self.b.mul(b).as_ref(), None)
    }
}

/*------------------------------------------------------*
| ModNode
*-------------------------------------------------------*/

pub struct ModNode {
    a: Box<dyn Node>,
    b: Box<dyn Node>,
    min: isize,
    max: isize,
}

impl ModNode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(a: &dyn Node, b: &dyn Node) -> Box<dyn Node> {
        let mut node = DivNode {
            a: a.clone(),
            b: b.clone(),
            min: 0,
            max: 0,
        };

        let (min, max) = node.get_bounds().unwrap();
        node.min = min;
        node.max = max;

        Box::new(node)
    }
}

impl Node for ModNode {
    fn min(&self) -> Option<isize> {
        Some(self.min)
    }

    fn max(&self) -> Option<isize> {
        Some(self.max)
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };

        format!(
            "{}{}%{}{}",
            lparen,
            self.a().unwrap().render(debug, strip_parens),
            self.b().unwrap().render(debug, strip_parens),
            rparen
        )
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(ModNode {
            a: self.a.clone(),
            b: self.b.clone(),
            min: self.min,
            max: self.max,
        })
    }

    fn get_bounds(&self) -> Option<(isize, isize)> {
        debug_assert!(self.a.min().unwrap() >= 0);
        debug_assert!(self.b().unwrap().is_num());

        let b = self.b.intval().unwrap();
        let (a_min, a_max) = (self.min().unwrap(), self.max().unwrap());
        let self_b_intval = self.b.intval().unwrap();

        if a_max - a_min >= self_b_intval || (a_min != a_max && a_min % b >= a_max % b) {
            Some((0, self_b_intval - 1))
        } else {
            Some((a_min % self_b_intval, a_max % self_b_intval))
        }
    }

    fn floordiv(&self, _b: &dyn Node, _: Option<bool>) -> Box<dyn Node> {
        todo!("ModNode::floordiv")
    }
}

/*------------------------------------------------------*
| SumNode
*-------------------------------------------------------*/

struct SumNode {
    nodes: Vec<Box<dyn Node>>,
    min: isize,
    max: isize,
}

impl SumNode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(nodes: &[Box<dyn Node>]) -> Box<dyn Node> {
        let node = SumNode {
            nodes: nodes.iter().map(|x| (*x).clone()).collect(),
            min: nodes.iter().map(|x| x.min().unwrap()).sum(),
            max: nodes.iter().map(|x| x.max().unwrap()).sum(),
        };

        Box::new(node)
    }
}

impl Node for SumNode {
    fn min(&self) -> Option<isize> {
        Some(self.min)
    }

    fn max(&self) -> Option<isize> {
        Some(self.max)
    }

    fn is_sum(&self) -> bool {
        true
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };

        let rendered_nodes = self
            .nodes
            .iter()
            .map(|x| x.render(debug, strip_parens))
            .collect::<Vec<_>>()
            .join("+");

        if strip_parens {
            rendered_nodes
        } else {
            format!("{}{}{}", lparen, rendered_nodes, rparen)
        }
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(SumNode {
            nodes: self.nodes.iter().map(|x| (*x).clone()).collect(),
            min: self.min,
            max: self.max,
        })
    }

    fn nodes(&self) -> Vec<&dyn Node> {
        self.nodes.iter().map(|x| x.as_ref()).collect()
    }

    fn mul(&self, b: &dyn Node) -> Box<dyn Node> {
        let nodes = self.nodes.iter().map(|x| x.mul(b)).collect::<Vec<_>>();
        sum(nodes
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<_>>()
            .as_slice())
    }

    fn floordiv(&self, _b: &dyn Node, facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        let facatoring_allowed = facatoring_allowed.unwrap_or(true);

        //         fully_divided: List[Node] = []
        // rest: List[Node] = []
        // if isinstance(b, SumNode):
        //   nu_num = sum(node.b for node in self.flat_components if node.__class__ is NumNode)
        //   de_num = sum(node.b for node in b.flat_components if node.__class__ is NumNode)
        //   if nu_num > 0 and de_num and (d:=nu_num//de_num) > 0: return NumNode(d) + (self-b*d) // b
        // if isinstance(b, Node):
        //   for x in self.flat_components:
        //     if x % b == 0: fully_divided.append(x // b)
        //     else: rest.append(x)
        //   if (sum_fully_divided:=create_rednode(SumNode, fully_divided)) != 0: return sum_fully_divided + create_rednode(SumNode, rest) // b
        //   return Node.__floordiv__(self, b, False)
        // if b == 1: return self
        // if not factoring_allowed: return Node.__floordiv__(self, b, factoring_allowed)
        // fully_divided, rest = [], []
        // _gcd = b
        // divisor = 1
        // for x in self.flat_components:
        //   if x.__class__ in (NumNode, MulNode):
        //     if x.b%b == 0: fully_divided.append(x//b)
        //     else:
        //       rest.append(x)
        //       _gcd = gcd(_gcd, x.b)
        //       if x.__class__ == MulNode and divisor == 1 and b%x.b == 0: divisor = x.b
        //   else:
        //     rest.append(x)
        //     _gcd = 1
        // if _gcd > 1: return Node.sum(fully_divided) + Node.sum(rest).__floordiv__(_gcd) // (b//_gcd)
        // if divisor > 1: return Node.sum(fully_divided) + Node.sum(rest).__floordiv__(divisor) // (b//divisor)
        // return Node.sum(fully_divided) + Node.__floordiv__(Node.sum(rest), b)

        todo!("SumNode::floordiv")
    }

    fn modulus(&self, _b: &dyn Node) -> Box<dyn Node> {
        todo!("SumNode::modulus")
    }
}

/*------------------------------------------------------*
| AndNode
*-------------------------------------------------------*/

struct AndNode {
    nodes: Vec<Box<dyn Node>>,
    min: isize,
    max: isize,
}

impl AndNode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(nodes: &[Box<dyn Node>]) -> Box<dyn Node> {
        let node = AndNode {
            nodes: nodes.iter().map(|x| (*x).clone()).collect(),
            min: nodes.iter().map(|x| x.min().unwrap()).min().unwrap_or(0),
            max: nodes.iter().map(|x| x.max().unwrap()).max().unwrap_or(0),
        };

        create_node(&node)
    }
}

impl Node for AndNode {
    fn min(&self) -> Option<isize> {
        Some(self.min)
    }

    fn max(&self) -> Option<isize> {
        Some(self.max)
    }

    fn is_and(&self) -> bool {
        true
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };

        let rendered_nodes = self
            .nodes
            .iter()
            .map(|x| x.render(debug, strip_parens))
            .collect::<Vec<_>>()
            .join(" and ");

        if strip_parens {
            rendered_nodes
        } else {
            format!("{}{}{}", lparen, rendered_nodes, rparen)
        }
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(AndNode {
            nodes: self.nodes.iter().map(|x| (*x).clone()).collect(),
            min: self.min,
            max: self.max,
        })
    }

    fn nodes(&self) -> Vec<&dyn Node> {
        self.nodes.iter().map(|x| x.as_ref()).collect()
    }

    fn floordiv(&self, b: &dyn Node, facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        let nodes = self
            .nodes
            .iter()
            .map(|x| x.floordiv(b, facatoring_allowed))
            .collect::<Vec<_>>();

        let nodes = nodes.iter().map(|x| x.as_ref()).collect::<Vec<_>>();

        ands(&nodes)
    }
}
