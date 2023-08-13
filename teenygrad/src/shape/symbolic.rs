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
    collections::BTreeMap,
    fmt::{Debug, Display},
    hash::Hash,
    ops,
};

use crate::helpers::{gcd, partition};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeAttrs {
    pub b: isize,
    pub min: isize,
    pub max: isize,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    Var {
        expr: String,
        val: Option<isize>,
        attrs: NodeAttrs,
    },
    Num {
        attrs: NodeAttrs,
    },
    LessThan {
        a: Box<Node>,
        attrs: NodeAttrs,
    },
    Mult {
        a: Box<Node>,
        attrs: NodeAttrs,
    },
    Div {
        a: Box<Node>,
        attrs: NodeAttrs,
    },
    Mod {
        a: Box<Node>,
        attrs: NodeAttrs,
    },
    Sum {
        nodes: Vec<Node>,
        attrs: NodeAttrs,
    },
    And {
        nodes: Vec<Node>,
        attrs: NodeAttrs,
    },
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_nodes(
            nodes: &[Node],
            separator: &str,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            f.write_str("(").unwrap();
            for (count, node) in nodes.iter().enumerate() {
                if count != 0 {
                    f.write_str(separator)?;
                }
                write!(f, "{}", node)?;
            }
            f.write_str(")").unwrap();
            Ok(())
        }

        match self {
            Node::Var { expr, .. } => write!(f, "{}", expr)?,
            Node::Num { attrs } => write!(f, "{}", attrs.b)?,
            Node::LessThan { a, attrs } => write!(f, "({}<{})", a, attrs.b)?,
            Node::Mult { a, attrs } => write!(f, "({}*{})", a, attrs.b)?,
            Node::Div { a, attrs } => write!(f, "({}//{})", a, attrs.b)?,
            Node::Mod { a, attrs } => write!(f, "({}%{})", a, attrs.b)?,
            Node::Sum { nodes, .. } => write_nodes(nodes, "+", f)?,
            Node::And { nodes, .. } => write_nodes(nodes, " and ", f)?,
        };

        Ok(())
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_nodes(
            nodes: &[Node],
            separator: &str,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            f.write_str("(").unwrap();
            for (count, node) in nodes.iter().enumerate() {
                if count != 0 {
                    f.write_str(separator)?;
                }
                write!(f, "{:?}", node)?;
            }
            f.write_str(")").unwrap();
            Ok(())
        }

        match self {
            Node::Var { expr, val, attrs } => {
                write!(f, "{:?}{:?}[{:?}-{:?}]", expr, val, attrs.min, attrs.max)?
            }
            Node::Num { attrs } => write!(f, "{:?}", attrs.b)?,
            Node::LessThan { a, attrs } => write!(f, "({:?}<{:?})", a, attrs.b)?,
            Node::Mult { a, attrs } => write!(f, "({:?}*{:?})", a, attrs.b)?,
            Node::Div { a, attrs } => write!(f, "({:?}//{:?})", a, attrs.b)?,
            Node::Mod { a, attrs } => write!(f, "({:?}%{:?})", a, attrs.b)?,
            Node::Sum { nodes, .. } => write_nodes(nodes, "+", f)?,
            Node::And { nodes, .. } => write_nodes(nodes, " and ", f)?,
        };

        Ok(())
    }
}

impl Node {
    pub fn new_num(b: isize) -> Node {
        Node::Num {
            attrs: NodeAttrs { b, min: b, max: b },
        }
    }

    pub fn new_node(node: Node) -> Node {
        let (min, max) = node.min_max();
        debug_assert!(min <= max);

        if min == max {
            Node::new_num(min)
        } else {
            node
        }
    }

    pub fn new_var(expr: &str, min: isize, max: isize) -> Node {
        // debug_assert!(min >= 0 && min <= max);

        if min == max {
            Node::new_num(min)
        } else {
            Node::Var {
                expr: expr.to_string(),
                val: None,
                attrs: NodeAttrs { b: 0, min, max },
            }
        }
    }

    pub fn new_lt(node: Node, b: isize) -> Node {
        let mut lt_node = Node::LessThan {
            a: Box::new(node),
            attrs: NodeAttrs { b, min: 0, max: 0 },
        };

        lt_node.update_min_max();
        lt_node
    }

    pub fn new_mult(node: Node, b: isize) -> Node {
        let mut mult_node = Node::Mult {
            a: Box::new(node),
            attrs: NodeAttrs { b, min: 0, max: 0 },
        };

        mult_node.update_min_max();
        mult_node
    }

    pub fn new_div(node: Node, b: isize) -> Node {
        let mut div_node = Node::Div {
            a: Box::new(node),
            attrs: NodeAttrs { b, min: 0, max: 0 },
        };

        div_node.update_min_max();
        div_node
    }

    pub fn new_mod(node: Node, b: isize) -> Node {
        let mut mod_node = Node::Mod {
            a: Box::new(node),
            attrs: NodeAttrs { b, min: 0, max: 0 },
        };

        mod_node.update_min_max();
        mod_node
    }

    pub fn new_sum(nodes: &[Node]) -> Node {
        let non_zero_nodes: Vec<&Node> = nodes
            .iter()
            .filter(|node| {
                let (min, max) = node.min_max();
                min != 0 || max != 0
            })
            .collect();

        match non_zero_nodes.len() {
            0 => Node::new_num(0),
            1 => non_zero_nodes[0].clone(),
            _ => {
                let mut new_nodes: Vec<Node> = vec![];
                let mut num_node_sum = 0;

                for node in non_zero_nodes {
                    match node {
                        Node::Num { attrs } => num_node_sum += attrs.b,
                        Node::Sum { .. } => {
                            for sub_node in node.flat_components() {
                                match sub_node {
                                    Node::Num { attrs } => num_node_sum += attrs.b,
                                    _ => new_nodes.push(sub_node.clone()),
                                }
                            }
                        }
                        _ => new_nodes.push(node.clone()),
                    }
                }

                let mut flat_nodes: Vec<&Node> = vec![];
                for node in new_nodes.iter() {
                    let a = match node {
                        Node::Mult { a, .. } => a,
                        _ => node,
                    };
                    if !flat_nodes.contains(&a) {
                        flat_nodes.push(a);
                    }
                }

                if new_nodes.len() > 1 && flat_nodes.len() < new_nodes.len() {
                    new_nodes = factorize(&new_nodes);
                }

                if num_node_sum != 0 {
                    new_nodes.push(Node::new_num(num_node_sum));
                }

                match new_nodes.len() {
                    0 => Node::new_num(0),
                    1 => new_nodes[0].clone(),
                    _ => {
                        let mut node = Node::Sum {
                            nodes: new_nodes,
                            attrs: NodeAttrs::default(),
                        };
                        node.update_min_max();
                        node
                    }
                }
            }
        }
    }

    pub fn new_ands(nodes: &[Node]) -> Node {
        if nodes.is_empty() {
            return Node::new_num(1);
        }

        if nodes.len() == 1 {
            return nodes[0].clone();
        }

        if nodes.iter().any(|x| {
            let (min, max) = x.min_max();
            min == 0 && max == 0
        }) {
            return Node::new_num(0);
        }

        let filtered_nodes = nodes
            .iter()
            .filter(|x| {
                let (min, max) = x.min_max();
                min != max
            })
            .cloned()
            .collect::<Vec<_>>();

        match filtered_nodes.len() {
            0 => Node::new_num(1),
            1 => filtered_nodes[0].clone(),
            _ => {
                let mut node = Node::And {
                    nodes: filtered_nodes,
                    attrs: NodeAttrs::default(),
                };
                node.update_min_max();
                node
            }
        }
    }

    pub fn attrs(&self) -> &NodeAttrs {
        match self {
            Node::Var { attrs, .. } => attrs,
            Node::Num { attrs } => attrs,
            Node::LessThan { attrs, .. } => attrs,
            Node::Mult { attrs, .. } => attrs,
            Node::Div { attrs, .. } => attrs,
            Node::Mod { attrs, .. } => attrs,
            Node::Sum { attrs, .. } => attrs,
            Node::And { attrs, .. } => attrs,
        }
    }

    pub fn attrs_mut(&mut self) -> &mut NodeAttrs {
        match self {
            Node::Var { attrs, .. } => attrs,
            Node::Num { attrs } => attrs,
            Node::LessThan { attrs, .. } => attrs,
            Node::Mult { attrs, .. } => attrs,
            Node::Div { attrs, .. } => attrs,
            Node::Mod { attrs, .. } => attrs,
            Node::Sum { attrs, .. } => attrs,
            Node::And { attrs, .. } => attrs,
        }
    }

    pub fn get_bounds(&self) -> (isize, isize) {
        match self {
            Node::LessThan { a, attrs } => {
                let (min, max) = a.min_max();
                let bounds_0 = if max < attrs.b { 1 } else { 0 };
                let bounds_1 = if min < attrs.b { 1 } else { 0 };

                (bounds_0, bounds_1)
            }
            Node::Mult { a, attrs } => {
                let (min, max) = a.min_max();
                if attrs.b >= 0 {
                    (min * attrs.b, max * attrs.b)
                } else {
                    (max * attrs.b, min * attrs.b)
                }
            }
            Node::Div { a, attrs } => {
                let (a_min, a_max) = a.min_max();
                debug_assert!(a_min >= 0);

                (a_min / attrs.b, a_max / attrs.b)
            }
            Node::Mod { a, attrs } => {
                let (a_min, a_max) = a.min_max();
                debug_assert!(a_min >= 0);

                if a_max - a_min >= attrs.b
                    || (a_min != a_max && a_min % attrs.b >= a_max % attrs.b)
                {
                    (0, attrs.b - 1)
                } else {
                    (a_min % attrs.b, a_max % attrs.b)
                }
            }
            Node::Sum { nodes, .. } => {
                let min = nodes.iter().map(|node| node.min_max().0).sum();
                let max = nodes.iter().map(|node| node.min_max().1).sum();
                (min, max)
            }
            Node::And { nodes, .. } => {
                let min = nodes.iter().map(|node| node.min_max().0).min().unwrap_or(0);
                let max = nodes.iter().map(|node| node.min_max().1).max().unwrap_or(0);
                (min, max)
            }
            _ => panic!("get_bounds: unsupported node type"),
        }
    }

    pub fn min_max(&self) -> (isize, isize) {
        let attrs = self.attrs();
        (attrs.min, attrs.max)
    }

    pub fn flat_components(&self) -> Vec<&Node> {
        match self {
            Node::Sum { nodes, .. } => {
                let mut new_nodes = Vec::new();
                for node in nodes {
                    new_nodes.extend(node.flat_components());
                }
                new_nodes
            }
            _ => vec![self],
        }
    }

    fn update_min_max(&mut self) -> &mut Self {
        let bounds = self.get_bounds();
        let attrs = self.attrs_mut();
        attrs.min = bounds.0;
        attrs.max = bounds.1;
        self
    }
}

/*------------------------------------------*
|  Add
*-------------------------------------------*/

impl ops::Add<isize> for Node {
    type Output = Node;

    fn add(self, rhs: isize) -> Self::Output {
        Node::new_sum(&[self, Node::new_num(rhs)])
    }
}

impl ops::Add<Node> for Node {
    type Output = Node;

    fn add(self, rhs: Node) -> Self::Output {
        if let Node::Num { attrs } = &rhs {
            if attrs.b == 0 {
                return self;
            }
        }

        Node::new_sum(&[self, rhs])
    }
}

/*------------------------------------------*
|  Ands
*-------------------------------------------*/

pub fn ands(_nodes: &[Node]) -> Node {
    // @staticmethod
    // def ands(nodes:List[Node]) -> Node:
    //   if not nodes: return NumNode(1)
    //   if len(nodes) == 1: return nodes[0]
    //   if any(x.min == x.max == 0 for x in nodes): return NumNode(0)

    //   # filter 1s
    //   nodes = [x for x in nodes if x.min != x.max]
    //   return create_rednode(AndNode, nodes) if len(nodes) > 1 else (nodes[0] if len(nodes) == 1 else NumNode(1))
    todo!()
}

/*------------------------------------------*
|  Factorize
*-------------------------------------------*/

pub fn factorize(nodes: &[Node]) -> Vec<Node> {
    let mut mul_groups = BTreeMap::<&Node, isize>::new();
    for x in nodes.iter() {
        let (a, b) = match x {
            Node::Mult { a, attrs } => (a.as_ref(), attrs.b),
            _ => (x, 1),
        };
        mul_groups.insert(a, mul_groups.get(a).unwrap_or(&0) + b);
    }

    mul_groups
        .iter()
        .filter(|group| *group.1 != 0)
        .map(|group| {
            if *group.1 != 1 {
                Node::new_mult((**group.0).clone(), *group.1)
            } else {
                (**group.0).clone()
            }
        })
        .collect()
}

/*------------------------------------------*
|  Floordiv
*-------------------------------------------*/

impl Node {
    pub fn floordiv(self, b: isize, factoring_allowed: Option<bool>) -> Node {
        match &self {
            Node::Var { .. } => node_floordiv(self.clone(), b, factoring_allowed),
            Node::Num { .. } => node_floordiv(self.clone(), b, factoring_allowed),
            Node::LessThan { a, attrs } => lt_floordiv((**a).clone(), b, attrs),
            Node::Mult { a, attrs, .. } => mult_floordiv(self.clone(), (**a).clone(), b, attrs),
            Node::Div { a, attrs } => div_floordiv((**a).clone(), b, attrs),
            Node::Mod { a, attrs } => mod_floordiv((**a).clone(), b, attrs),
            Node::Sum { .. } => sum_floordiv(self.clone(), b, factoring_allowed),
            Node::And { nodes, .. } => and_floordiv(nodes, b),
        }
    }
}

fn node_floordiv(node: Node, b: isize, factoring_allowed: Option<bool>) -> Node {
    debug_assert!(b != 0);

    if b < 0 {
        return node.floordiv(-b, factoring_allowed) * -1;
    }

    if b == 1 {
        return node;
    }

    let (min, _) = node.min_max();
    if min < 0 {
        let offset = (min as f32 / b as f32).floor() as isize;

        (node + (-offset * b)).floordiv(b, Some(false)) + offset
    } else {
        Node::new_node(Node::new_div(node, b))
    }
}

fn lt_floordiv(a: Node, b: isize, attrs: &NodeAttrs) -> Node {
    a.floordiv(b, None).lt(attrs.b / b)
}

fn mult_floordiv(node: Node, a: Node, b: isize, attrs: &NodeAttrs) -> Node {
    if attrs.b % b == 0 {
        a * (attrs.b / b)
    } else if (b % attrs.b == 0) && (attrs.b > 0) {
        a.floordiv(b / attrs.b, None)
    } else {
        node_floordiv(node, b, None)
    }
}

fn div_floordiv(node: Node, b: isize, attrs: &NodeAttrs) -> Node {
    node.floordiv(attrs.b * b, None)
}

fn mod_floordiv(node: Node, b: isize, attrs: &NodeAttrs) -> Node {
    if attrs.b % b == 0 {
        node.floordiv(b, None).modulus(attrs.b / b)
    } else {
        node_floordiv(node, b, None)
    }
}

fn sum_floordiv(node: Node, b: isize, factoring_allowed: Option<bool>) -> Node {
    if b == 1 {
        return node.to_owned();
    }

    if !factoring_allowed.unwrap_or(true) {
        return node_floordiv(node, b, factoring_allowed);
    }

    let mut fully_divided: Vec<Node> = vec![];
    let mut rest: Vec<Node> = vec![];
    let mut _gcd = b;
    let mut divisor = 1;

    for x in node.flat_components() {
        match x {
            Node::Num { attrs } | Node::Mult { attrs, .. } => {
                if attrs.b % b == 0 {
                    fully_divided.push(x.clone().floordiv(b, None))
                } else {
                    rest.push(x.clone());
                    _gcd = gcd(_gcd, attrs.b);
                    if matches!(x, Node::Mult { .. }) && divisor == 1 && b % attrs.b == 0 {
                        divisor = attrs.b
                    }
                }
            }
            _ => {
                rest.push(x.clone());
                _gcd = 1;
            }
        }
    }

    if _gcd > 1 {
        sum(&fully_divided) + sum(&rest).floordiv(_gcd, None).floordiv(b / _gcd, None)
    } else if divisor > 1 {
        sum(&fully_divided)
            + sum(&rest)
                .floordiv(divisor, None)
                .floordiv(b / divisor, None)
    } else {
        let rest_sum = sum(&rest);
        sum(&fully_divided) + node_floordiv(rest_sum.clone(), b, None)
    }
}

fn and_floordiv(nodes: &[Node], b: isize) -> Node {
    let x: Vec<Node> = nodes.iter().map(|x| x.clone().floordiv(b, None)).collect();

    Node::new_ands(&x)
}

/*------------------------------------------*
|  GE
*-------------------------------------------*/

impl Node {
    pub fn ge(self, b: isize) -> Node {
        (-self).lt(-b + 1)
    }
}

/*------------------------------------------*
|  LT
*-------------------------------------------*/

impl Node {
    pub fn lt(self, b: isize) -> Node {
        let mut lhs = None;
        if let Node::Sum { nodes, .. } = &self {
            lhs = sum_lt(nodes, b);
        }

        Node::new_node(Node::new_lt(lhs.unwrap_or(self), b))
    }
}

fn sum_lt(nodes: &[Node], b: isize) -> Option<Node> {
    fn get_mult_value(node: &Node) -> Option<isize> {
        match node {
            Node::Mult { attrs, .. } => Some(attrs.b),
            _ => None,
        }
    }
    fn compute_gcd(a: isize, node: &Node) -> isize {
        gcd(a, get_mult_value(node).unwrap())
    }

    let (muls, others) = partition(nodes, |node: &Node| -> bool {
        if let Node::Mult { ref attrs, .. } = node {
            attrs.b > 0 && attrs.max >= b
        } else {
            false
        }
    });
    if !muls.is_empty() {
        let mut mul_gcd = get_mult_value(muls.get(0).unwrap()).unwrap();
        mul_gcd = muls[1..]
            .iter()
            .fold(mul_gcd, |acc: isize, node| compute_gcd(acc, node));
        if b % mul_gcd == 0 {
            let others_owned: Vec<Node> = others.iter().map(|n| (**n).clone()).collect();
            let all_others = sum(&others_owned);
            let (min, max) = all_others.min_max();
            if min >= 0 && max < mul_gcd {
                let muls_owned: Vec<Node> = muls.iter().map(|n| (**n).clone()).collect();
                Some(sum(&muls_owned))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

/*------------------------------------------*
|  Modulus
*-------------------------------------------*/

impl Node {
    pub fn modulus(&self, b: isize) -> Node {
        match self {
            Node::Mult { a, attrs, .. } => mod_mult(a, b, attrs),
            Node::Sum { nodes, .. } => mod_sum(nodes, b),
            _ => mod_node(self, b),
        }
    }
}

fn mod_node(node: &Node, b: isize) -> Node {
    debug_assert!(b > 0);

    if b == 1 {
        return Node::new_num(0);
    }

    let (min, max) = node.min_max();
    if min >= 0 && max < b {
        return node.clone();
    }

    if min < 0 {
        let x = (min as f32 / b as f32).floor() * b as f32;
        return (node.clone() - x as isize).modulus(b);
    }

    Node::new_mod(node.clone(), b)
}

fn mod_mult(a: &Node, b: isize, attrs: &NodeAttrs) -> Node {
    let a = a.clone() * (attrs.b % b);
    mod_node(&a, b)
}

fn mod_sum(nodes: &[Node], b: isize) -> Node {
    let mut new_nodes: Vec<Node> = vec![];

    for x in nodes {
        match x {
            Node::Num { attrs } => {
                new_nodes.push(Node::new_num(attrs.b % b));
            }
            Node::Mult { a, attrs } => {
                new_nodes.push(*a.clone() * (attrs.b % b));
            }
            _ => {
                new_nodes.push(x.clone());
            }
        }
    }

    mod_node(&sum(&new_nodes), b)
}

/*------------------------------------------*
|  Mul
*-------------------------------------------*/

impl ops::Mul<isize> for Node {
    type Output = Node;
    fn mul(self, b: isize) -> Self::Output {
        match self.clone() {
            Node::LessThan { a, attrs } => (*a * b).lt(attrs.b * b),
            Node::Mult { a, attrs } => *a * (attrs.b * b),
            Node::Sum { nodes, .. } => {
                let _nodes: Vec<Node> = nodes.iter().map(|node| node.clone() * b).collect();
                sum(&_nodes)
            }
            Node::And { nodes, .. } => {
                let _nodes: Vec<Node> = nodes.iter().map(|node| node.clone() * b).collect();
                ands(&_nodes)
            }
            _ => match b {
                0 => Node::new_num(0),
                1 => self,
                _ => Node::new_mult(self, b),
            },
        }
    }
}

/*------------------------------------------*
|  Neg
*-------------------------------------------*/

impl ops::Neg for Node {
    type Output = Node;

    fn neg(self) -> Self::Output {
        self * -1
    }
}

/*------------------------------------------*
|  Sub
*-------------------------------------------*/

impl ops::Sub<isize> for Node {
    type Output = Node;

    fn sub(self, rhs: isize) -> Self::Output {
        self + (-rhs)
    }
}

impl ops::Sub<Node> for Node {
    type Output = Node;

    fn sub(self, rhs: Node) -> Self::Output {
        self + (-rhs)
    }
}

/*------------------------------------------*
|  Sum
*-------------------------------------------*/

pub fn sum(nodes: &[Node]) -> Node {
    fn flat_components(nodes: &[Node]) -> Vec<Node> {
        let mut new_nodes = vec![];
        for node in nodes.iter() {
            if let Node::Sum { nodes, .. } = node {
                new_nodes.extend(flat_components(&nodes[..]));
            }
        }
        new_nodes
    }

    let all_nodes: Vec<&Node> = nodes
        .iter()
        .filter(|node| {
            let (min, max) = node.min_max();
            (min != 0) || (max != 0)
        })
        .collect();
    let mut node_sum: isize = 0;
    let mut new_nodes: Vec<Node> = vec![];

    if all_nodes.is_empty() {
        return Node::new_num(0);
    }

    for node in all_nodes {
        match node {
            Node::Num { attrs } => node_sum += attrs.b,
            Node::Sum { nodes, .. } => {
                let sub_nodes = flat_components(nodes);
                for sub_node in sub_nodes.iter() {
                    if let Node::Num { attrs } = sub_node {
                        node_sum += attrs.b;
                    } else {
                        new_nodes.push(sub_node.clone());
                    }
                }
            }
            _ => new_nodes.push((*node).clone()),
        }

        let num_mult_nodes = new_nodes
            .iter()
            .filter(|node| matches!(node, Node::Mult { .. }))
            .count();

        if num_mult_nodes < new_nodes.len() {
            new_nodes = factorize(&new_nodes[..]);
        }
    }

    if node_sum != 0 {
        new_nodes.push(Node::new_num(node_sum));
    }

    match new_nodes.len() {
        0 => Node::new_num(0),
        1 => new_nodes.get(0).unwrap().clone(),
        _ => Node::new_sum(&new_nodes),
    }
}

/*------------------------------------------*
|  Vars
*-------------------------------------------*/

impl Node {
    pub fn vars(&self) -> Vec<&Node> {
        match self {
            Node::Var { .. } => vec![self],
            Node::LessThan { a, .. } => a.vars(),
            Node::Num { .. } => vec![],
            Node::Mult { a, .. } => a.vars(),
            Node::Div { a, .. } => a.vars(),
            Node::Mod { a, .. } => a.vars(),
            Node::Sum { nodes, .. } => nodes.iter().flat_map(|node| node.vars()).collect(),
            Node::And { nodes, .. } => nodes.iter().flat_map(|node| node.vars()).collect(),
        }
    }
}
