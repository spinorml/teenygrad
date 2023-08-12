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

pub mod add;
pub mod ands;
pub mod factorize;
pub mod floordiv;
pub mod ge;
pub mod lt;
pub mod modulus;
pub mod mul;
pub mod neg;
pub mod sub;
pub mod sum;
pub mod vars;

use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::shape::symbolic::factorize::factorize;

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
