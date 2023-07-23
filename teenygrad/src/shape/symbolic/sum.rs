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

use crate::shape::symbolic::factorize::factorize;

use super::Node;

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
