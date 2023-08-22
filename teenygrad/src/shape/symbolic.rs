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

use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::Hasher,
};

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

    fn is_num(&self) -> bool {
        false
    }

    fn is_var(&self) -> bool {
        false
    }

    fn is_mul(&self) -> bool {
        false
    }

    fn intval(&self) -> isize {
        panic!("Invalid node")
    }

    fn flat_components(&self) -> Vec<Box<dyn Node>> {
        self.a()
            .map(|x| x.flat_components())
            .unwrap_or_default()
            .into_iter()
            .chain(
                self.b()
                    .map(|x| x.flat_components())
                    .unwrap_or_default()
                    .into_iter(),
            )
            .chain(vec![self.clone()].into_iter())
            .collect()
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
            match b.intval() {
                0 => return num(0),
                1 => return self.clone(),
                _ => {}
            }
        }

        if self.is_num() {
            if b.is_num() {
                return num(self.intval() * b.intval());
            }

            return b.mul(self.clone().as_ref());
        }

        if b.is_num() {
            return create_node(MulNode::new(self.clone().as_ref(), b.b().unwrap()).as_ref());
        }

        create_node(MulNode::new(self.clone().as_ref(), b).as_ref())
    }

    fn floordiv(&self, _other: &dyn Node, _facatoring_allowed: Option<bool>) -> Box<dyn Node> {
        todo!()
    }

    fn modulus(&self, _other: &dyn Node) -> Box<dyn Node> {
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

    fn lt(&self, _other: &dyn Node) -> Box<dyn Node> {
        todo!()
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
    todo!()
}

pub fn sum(nodes: &[&dyn Node]) -> Box<dyn Node> {
    let nodes = nodes
        .iter()
        .filter(|x| x.min() != 0 || x.max() != 0)
        .collect::<Vec<_>>();
    match nodes.len() {
        0 => return num(0),
        1 => return (*nodes[0]).clone(),
        _ => {}
    }

    // new_nodes: List[Node] = []
    // num_node_sum = 0
    let mut new_nodes: Vec<Box<dyn Node>> = vec![];
    let mut num_node_sum = 0;

    for node in nodes.iter().flat_map(|x| x.flat_components()) {
        if node.is_num() {
            num_node_sum += node.intval();
        } else {
            new_nodes.push(node);
        }
    }

    if !new_nodes.is_empty() {
        let y = new_nodes
            .iter()
            .map(|x| {
                if x.is_mul() {
                    x.a().unwrap().render(false, false)
                } else {
                    x.render(false, false)
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

    // return create_rednode(SumNode, new_nodes) if len(new_nodes) > 1 else new_nodes[0] if len(new_nodes) == 1 else NumNode(0)

    // if len(new_nodes) > 1 and len(set([x.a if isinstance(x, MulNode) else x for x in new_nodes])) < len(new_nodes):
    //   new_nodes = Node.factorize(new_nodes)
    // if num_node_sum: new_nodes.append(NumNode(num_node_sum))

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

    fn render(&self, _debug: bool, _strip_parens: bool) -> String {
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

    fn b(&self) -> Option<&dyn Node> {
        Some(self)
    }

    fn intval(&self) -> isize {
        self.value
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
}

/*------------------------------------------------------*
| OpNode
*-------------------------------------------------------*/

trait OpNode: Node {
    fn vars(&self) -> Vec<&dyn Node> {
        let mut vars = self.a().unwrap().vars();
        vars.extend(self.b().unwrap().vars());
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

    fn is_mul(&self) -> bool {
        true
    }

    fn render(&self, _debug: bool, _strip_parens: bool) -> String {
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
