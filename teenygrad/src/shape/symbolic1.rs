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

use std::{collections::hash_map::DefaultHasher, hash::Hasher};

pub trait Node {
    fn expr(&self) -> &str {
        panic!("Invalid node")
    }

    fn a(&self) -> &dyn Node {
        panic!("Invalid node")
    }

    fn b(&self) -> &dyn Node {
        panic!("Invalid node")
    }

    fn min(&self) -> isize;

    fn max(&self) -> isize;

    fn intval(&self) -> isize {
        panic!("Invalid node")
    }

    fn nodes(&self) -> Vec<&dyn Node> {
        panic!("Invalid node")
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String;

    fn clone(&self) -> Box<dyn Node>;

    fn key(&self) -> String {
        self.render(false, false)
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        hasher.write(self.key().as_bytes());
        hasher.finish()
    }

    fn eq(&self, other: &dyn Node) -> bool {
        self.key() == other.key()
    }

    fn as_bool(&self) -> bool {
        !(self.min() == self.max() && self.min() == 0)
    }

    fn vars(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn get_bounds(&self) -> (isize, isize) {
        panic!("Invalid node")
    }

    fn flat_components(&self) -> Vec<&dyn Node> {
        panic!("Invalid node")
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

    fn mul(&self, other: &dyn Node) -> Box<dyn Node> {
        todo!()
    }

    fn floordiv(&self, other: &dyn Node, facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        todo!()
    }

    fn modulus(&self, other: &dyn Node) -> Box<dyn Node> {
        todo!()
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

    fn lt(&self, other: &dyn Node) -> Box<dyn Node> {
        todo!()
    }
}

/*------------------------------------------------------*
| Utility functions
*-------------------------------------------------------*/

pub fn num(value: isize) -> Box<dyn Node> {
    NumNode::new(value)
}

pub fn factorize(_nodes: &[&dyn Node]) -> Box<dyn Node> {
    todo!()
}

pub fn sum(_nodes: &[&dyn Node]) -> Box<dyn Node> {
    todo!()
}

pub fn ands(_nodes: &[&dyn Node]) -> Box<dyn Node> {
    todo!()
}

pub fn create_node(node: &dyn Node) -> Box<dyn Node> {
    if node.min() == node.max() {
        num(node.min())
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
        Box::new(Var {
            expr: expr.to_string(),
            min,
            max,
        })
    }
}

impl Node for Var {
    fn expr(&self) -> &str {
        &self.expr
    }

    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        todo!()
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
    fn min(&self) -> isize {
        self.value
    }

    fn max(&self) -> isize {
        self.value
    }

    fn intval(&self) -> isize {
        self.value
    }

    fn render(&self, _debug: bool, _strip_parens: bool) -> String {
        self.value.to_string()
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(NumNode { value: self.value })
    }
}

/*------------------------------------------------------*
| OpNode
*-------------------------------------------------------*/

trait OpNode: Node {
    fn vars(&self) -> Vec<&dyn Node> {
        let mut vars = self.a().vars();
        vars.extend(self.b().vars());
        vars
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

        let (min, max) = node.get_bounds();
        node.min = min;
        node.max = max;

        Box::new(node)
    }
}

impl Node for LtNode {
    fn a(&self) -> &dyn Node {
        self.a.as_ref()
    }

    fn b(&self) -> &dyn Node {
        self.b.as_ref()
    }

    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
    }

    fn render(&self, _debug: bool, _strip_parens: bool) -> String {
        todo!()
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(LtNode {
            a: self.a.clone(),
            b: self.b.clone(),
            min: self.min,
            max: self.max,
        })
    }

    fn get_bounds(&self) -> (isize, isize) {
        if self.a.max() < self.b.min() {
            (1, 1)
        } else if self.a.min() > self.b.max() {
            (0, 0)
        } else {
            (0, 1)
        }
    }

    fn floordiv(&self, b: &dyn Node, _: Option<bool>) -> Box<dyn Node> {
        let x = self.a.floordiv(self.b.as_ref(), None);
        let y = self.b.floordiv(b, None);
        x.lt(y.as_ref())
    }
}

impl OpNode for LtNode {}

/*------------------------------------------------------*
| MulNode
*-------------------------------------------------------*/

struct MulNode {
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

        let (min, max) = node.get_bounds();
        node.min = min;
        node.max = max;

        Box::new(node)
    }
}

impl Node for MulNode {
    fn min(&self) -> isize {
        self.min
    }

    fn max(&self) -> isize {
        self.max
    }

    fn render(&self, debug: bool, strip_parens: bool) -> String {
        todo!()
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
        self.a.mul(b).lt(self.b.mul(b).as_ref())
    }

    fn floordiv(&self, _other: &dyn Node, _facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        todo!()
    }

    fn modulus(&self, _other: &dyn Node) -> Box<dyn Node> {
        todo!()
    }

    fn get_bounds(&self) -> (isize, isize) {
        let b = self.b.intval();

        if b >= 0 {
            (self.a.min() * b, self.a.max() * b)
        } else {
            (self.a.max() * b, self.a.min() * b)
        }
    }
}

impl OpNode for MulNode {}

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

        let (min, max) = node.get_bounds();
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

    fn render(&self, _debug: bool, _strip_parens: bool) -> String {
        todo!()
    }

    fn clone(&self) -> Box<dyn Node> {
        Box::new(DivNode {
            a: self.a.clone(),
            b: self.b.clone(),
            min: self.min,
            max: self.max,
        })
    }

    fn get_bounds(&self) -> (isize, isize) {
        debug_assert!(self.a.min() >= 0);

        let b = self.b.intval();
        (self.a.min() / b, self.a.max() / b)
    }

    fn floordiv(&self, b: &dyn Node, _: Option<bool>) -> Box<dyn Node> {
        self.a.floordiv(self.b.mul(b).as_ref(), None)
    }
}

impl OpNode for DivNode {}

/*------------------------------------------------------*
| RedNode
*-------------------------------------------------------*/

pub trait RedNode: Node {
    fn init(&mut self, nodes: &[&dyn Node]);

    fn vars(&self) -> Vec<&dyn Node> {
        let mut vars = vec![];
        for node in self.nodes() {
            vars.extend(node.vars());
        }
        vars
    }
}
