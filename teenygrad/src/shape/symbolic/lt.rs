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

use crate::helpers::{gcd, partition};

use super::{sum::sum, Node};

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
