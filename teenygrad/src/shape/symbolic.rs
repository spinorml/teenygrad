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

    fn floordiv(&self, b: &dyn Node, _facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        if !b.is_num() {
            if self.key() == b.key() {
                return num(1);
            }

            if (b.sub(self.clone().as_ref()).min().unwrap() > 0) && (self.min().unwrap() >= 0) {
                return num(0);
            }

            panic!("Not supported: {}//{}", self.key(), b.key());
        }

        let b_val = b.intval().unwrap();

        if b_val < 0 {
            return self.floordiv(b.neg().as_ref(), None).neg();
        }

        if b_val == 1 {
            return self.clone();
        }

        if self.min().unwrap() < 0 {
            let offset = self.min().unwrap() / b_val;
            return self
                .add(num(-offset * b_val).as_ref())
                .floordiv(b, None)
                .add(num(offset).as_ref());
        }

        create_node(DivNode::new(self.clone().as_ref(), b).as_ref())
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

pub fn ands(_nodes: &[&dyn Node]) -> Box<dyn Node> {
    todo!("ands")
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
        let _k1 = node.key();

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
            self.a().unwrap().render(debug, strip_parens),
            self.b().unwrap().render(debug, strip_parens),
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
            print!("LtNode::get_bounds: {}", self.key());
            print!("LtNode::a: {}", self.a.key());
            print!("LtNode::a-min: {:?}", self.a.min());
            let a_min = self.a.min().unwrap();
            let a_max = self.a.max().unwrap();
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
        let x = self.a.floordiv(self.b.as_ref(), None);
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

    fn floordiv(&self, _other: &dyn Node, _facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        // if self.b % b == 0: return self.a*(self.b//b)
        // if b % self.b == 0 and self.b > 0: return self.a//(b//self.b)
        // return Node.__floordiv__(self, b, factoring_allowed)
        todo!("MulNode::floordiv")
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
            self.a().unwrap().render(debug, strip_parens),
            self.b().unwrap().render(debug, strip_parens),
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
}
