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

use crate::{helpers::gcd, shape::symbolic::sum::sum};

use super::{Node, NodeAttrs};

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
