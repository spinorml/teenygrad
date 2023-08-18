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

use std::{fmt::Debug, fmt::Display, hash::Hash, ops::*};

pub enum NodeOrInt {
    Node(Box<dyn Node>),
    Int(isize),
}
pub trait Node {
    fn b(&self) -> NodeOrInt;
    fn min(&self) -> isize;
    fn max(&self) -> isize;

    fn display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    fn clone(&self) -> Box<dyn Node>;

    fn vars(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn add(&self, other: NodeOrInt) -> Box<dyn Node> {
        todo!()
    }

    fn neg(&self) -> Box<dyn Node> {
        todo!()
    }

    fn radd(&self, other: isize) -> Box<dyn Node> {
        self.add(NodeOrInt::Int(other))
    }

    fn sub(&self, other: NodeOrInt) -> Box<dyn Node> {
        let neg_value = match other {
            NodeOrInt::Node(other) => NodeOrInt::Node(other.neg()),
            NodeOrInt::Int(other) => NodeOrInt::Int(-other),
        };
        self.add(neg_value)
    }
}
