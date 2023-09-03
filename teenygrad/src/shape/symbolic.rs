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

use std::vec;

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

    fn min(&self) -> isize;

    fn max(&self) -> isize;

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
        !(self.min() == self.max() && self.min() == 0)
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

    fn simplify(&self) -> Box<dyn Node>;

    fn simplify_range(&self) -> Box<dyn Node> {
        debug_assert!(self.min() <= self.max());

        if self.min() == self.max() {
            num(self.min())
        } else {
            self.clone()
        }
    }

    fn neg(&self) -> Box<dyn Node> {
        self.mul(num(-1))
    }

    fn add(&self, other: Box<dyn Node>) -> Box<dyn Node> {
        let tmp1 = self.clone();

        sum(&[tmp1, other])
    }

    fn sub(&self, other: Box<dyn Node>) -> Box<dyn Node> {
        self.add(other.neg())
    }

    fn mul(&self, b: Box<dyn Node>) -> Box<dyn Node> {
        MulNode::new(self.clone(), b)
    }

    fn floordiv(&self, b: Box<dyn Node>, factoring_allowed: Option<bool>) -> Box<dyn Node> {
        DivNode::new(self.clone(), b, factoring_allowed.unwrap_or(true))
    }

    fn modulus(&self, b: Box<dyn Node>) -> Box<dyn Node> {
        ModNode::new(self.clone(), b)
    }

    fn le(&self, other: &dyn Node) -> Box<dyn Node> {
        self.lt(other.add(num(1)))
    }

    fn gt(&self, other: &dyn Node) -> Box<dyn Node> {
        self.neg().le(other.neg().as_ref())
    }

    fn ge(&self, other: Box<dyn Node>) -> Box<dyn Node> {
        self.neg().lt(other.neg().add(num(1)))
    }

    fn lt(&self, b: Box<dyn Node>) -> Box<dyn Node> {
        LtNode::new(self.clone(), b)
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
    Var::new(expr, min, max)
}

pub fn sum(nodes: &[Box<dyn Node>]) -> Box<dyn Node> {
    SumNode::new(nodes)
}

pub fn ands(nodes: &[Box<dyn Node>]) -> Box<dyn Node> {
    AndNode::new(nodes)
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

    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
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

    fn simplify(&self) -> Box<dyn Node> {
        self.simplify_range()
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
    fn min(&self) -> isize {
        self.value
    }

    fn max(&self) -> isize {
        self.value
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

    fn simplify(&self) -> Box<dyn Node> {
        self.simplify_range()
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
    pub fn new(a: Box<dyn Node>, b: Box<dyn Node>) -> Box<dyn Node> {
        let mut node = LtNode {
            a,
            b,
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

    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };
        let debug_str = if debug {
            format!("[{}, {}]", self.min, self.max)
        } else {
            "".to_string()
        };

        format!(
            "{lparen}{}<{}{rparen}{debug_str}",
            self.a.render(debug, strip_parens),
            self.b.render(debug, strip_parens),
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
            let (a_min, a_max) = (self.a.min(), self.a.max());
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

    fn vars(&self) -> Vec<&dyn Node> {
        let mut v: Vec<&dyn Node> = vec![];
        v.extend(self.a().unwrap().vars());
        v.extend(self.b().unwrap().vars());
        v
    }

    fn simplify(&self) -> Box<dyn Node> {
        let a = self.a.simplify();
        let b = self.b.simplify();

        LtNode::new(a, b).simplify_range()
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
    pub fn new(a: Box<dyn Node>, b: Box<dyn Node>) -> Box<dyn Node> {
        let mut node = MulNode {
            a,
            b,
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

    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
    }

    fn is_mul(&self) -> bool {
        true
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };
        let a = self.a();
        let b = self.b();
        let debug_str = if debug {
            format!("[{}, {}]", self.min, self.max)
        } else {
            "".to_string()
        };

        format!(
            "{lparen}{}*{}{rparen}{debug_str}",
            a.unwrap().render(debug, strip_parens),
            b.unwrap().render(debug, strip_parens),
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

    fn mul(&self, b: Box<dyn Node>) -> Box<dyn Node> {
        self.a.mul(b)
    }

    fn modulus(&self, b: Box<dyn Node>) -> Box<dyn Node> {
        self.a.modulus(b)
    }

    fn get_bounds(&self) -> Option<(isize, isize)> {
        let b = self.b.intval().unwrap();

        if b >= 0 {
            Some((self.a.min() * b, self.a.max() * b))
        } else {
            Some((self.a.max() * b, self.a.min() * b))
        }
    }

    fn simplify(&self) -> Box<dyn Node> {
        let a = self.a.simplify();
        let b = self.b.simplify();

        let node = if a.is_num() && b.is_num() {
            num(self.a.intval().unwrap() * self.b.intval().unwrap())
        } else {
            self.clone()
        };

        node.simplify_range()
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
    factoring_allowed: bool,
}

impl DivNode {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(a: Box<dyn Node>, b: Box<dyn Node>, factoring_allowed: bool) -> Box<dyn Node> {
        let mut node = DivNode {
            a,
            b,
            min: 0,
            max: 0,
            factoring_allowed,
        };

        let (min, max) = node.get_bounds().unwrap();
        node.min = min;
        node.max = max;

        Box::new(node)
    }
}

impl Node for DivNode {
    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
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
            factoring_allowed: self.factoring_allowed,
        })
    }

    fn get_bounds(&self) -> Option<(isize, isize)> {
        debug_assert!(self.a.min() >= 0);

        let b = self.b.intval().unwrap();
        Some((self.a.min() / b, self.a.max() / b))
    }

    fn simplify(&self) -> Box<dyn Node> {
        self.simplify_range()
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
    pub fn new(a: Box<dyn Node>, b: Box<dyn Node>) -> Box<dyn Node> {
        let mut node = ModNode {
            a,
            b,
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
    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
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
        debug_assert!(self.a.min() >= 0);
        debug_assert!(self.b().unwrap().is_num());

        let b = self.b.intval().unwrap();
        let (a_min, a_max) = (self.min(), self.max());
        let self_b_intval = self.b.intval().unwrap();

        if a_max - a_min >= self_b_intval || (a_min != a_max && a_min % b >= a_max % b) {
            Some((0, self_b_intval - 1))
        } else {
            Some((a_min % self_b_intval, a_max % self_b_intval))
        }
    }

    fn simplify(&self) -> Box<dyn Node> {
        self.simplify_range()
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
            min: nodes.iter().map(|x| x.min()).sum(),
            max: nodes.iter().map(|x| x.max()).sum(),
        };

        Box::new(node)
    }
}

impl Node for SumNode {
    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
    }

    fn is_sum(&self) -> bool {
        true
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        let lparen = if strip_parens { "" } else { "(" };
        let rparen = if strip_parens { "" } else { ")" };
        let debug_str = if debug {
            format!("[{}, {}]", self.min, self.max)
        } else {
            "".to_string()
        };

        let rendered_nodes = self
            .nodes
            .iter()
            .map(|x| x.render(debug, strip_parens))
            .collect::<Vec<_>>()
            .join("+");

        format!("{lparen}{}{rparen}{debug_str}", rendered_nodes)
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

    fn simplify(&self) -> Box<dyn Node> {
        let nodes = self.nodes.iter().map(|x| x.simplify()).collect::<Vec<_>>();
        let (num_nodes, other_nodes): (Vec<_>, Vec<_>) = nodes.iter().partition(|x| x.is_num());
        let num_sum = num_nodes.iter().map(|x| x.intval().unwrap()).sum::<isize>();

        let mut result = vec![];
        if num_sum != 0 {
            result.push(num(num_sum));
        }
        other_nodes.iter().for_each(|x| result.push((**x).clone()));

        match result.len() {
            0 => num(0),
            1 => result[0].simplify_range(),
            _ => SumNode::new(&result).simplify_range(),
        }
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
            min: nodes.iter().map(|x| x.min()).min().unwrap_or(0),
            max: nodes.iter().map(|x| x.max()).max().unwrap_or(0),
        };

        Box::new(node)
    }
}

impl Node for AndNode {
    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
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

    fn simplify(&self) -> Box<dyn Node> {
        self.simplify_range()
    }
}
