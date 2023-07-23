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

use std::ops::Mul;

use super::{sum::sum, Node, NodeAttrs};

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
                new_nodes.push(a.clone().mul(attrs.b % b));
            }
            _ => {
                new_nodes.push(x.clone());
            }
        }
    }

    mod_node(&sum(&new_nodes), b)
}
