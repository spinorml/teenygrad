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

use std::collections::BTreeMap;

use super::Node;

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
