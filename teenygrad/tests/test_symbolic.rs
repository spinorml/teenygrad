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

use teenygrad::shape::symbolic1::{num, Node, Var};

fn test_variable(v: &Box<dyn Node>, min: isize, max: isize, s: &str) {
    let (vmin, vmax) = (v.min(), v.max());

    assert_eq!(format!("{}", v.render(false, false)), s);
    assert_eq!(vmin, min);
    assert_eq!(vmax, max);
}

#[test]
fn test_ge() {
    let node = Var::new("a", 3, 8);

    test_variable(&node.ge(num(77).as_ref()), 0, 0, "0");
    test_variable(&node.ge(num(9).as_ref()), 0, 0, "0");
    test_variable(&node.ge(num(8).as_ref()), 0, 1, "((a*-1)<-7)");
    test_variable(&node.ge(num(4).as_ref()), 0, 1, "((a*-1)<-3)");
    test_variable(&node.ge(num(3).as_ref()), 1, 1, "1");
    test_variable(&node.ge(num(2).as_ref()), 1, 1, "1");
}

#[test]
fn test_lt() {
    let node = Var::new("a", 3, 8);

    test_variable(&node.lt(num(77).as_ref()), 1, 1, "1");
    test_variable(&node.lt(num(9).as_ref()), 1, 1, "1");
    test_variable(&node.lt(num(8).as_ref()), 0, 1, "(a<8)");
    test_variable(&node.lt(num(4).as_ref()), 0, 1, "(a<4)");
    test_variable(&node.lt(num(3).as_ref()), 0, 0, "0");
    test_variable(&node.lt(num(2).as_ref()), 0, 0, "0");
}

#[test]
fn test_ge_divides() {
    let expr = (Var::new("idx", 0, 511).mul(num(4).as_ref()))
        .add(Var::new("FLOAT4_INDEX", 0, 3).as_ref())
        .lt(num(512).as_ref());

    test_variable(&expr, 0, 1, "((idx*4)<512)");
    test_variable(
        &expr.floordiv(num(4).as_ref(), Some(false)),
        0,
        1,
        "(idx<128)",
    );
}

#[test]
fn test_ge_divides_and() {
    let expr = Node::new_ands(&[
        (Node::new_var("idx1", 0, 511) * 4 + Node::new_var("FLOAT4_INDEX", 0, 3)).lt(512),
        (Node::new_var("idx2", 0, 511) * 4 + Node::new_var("FLOAT4_INDEX", 0, 3)).lt(512),
    ]);
    let x = expr.floordiv(4, None);

    test_variable(x, 0, 1, "((idx1<128) and (idx2<128))");

    let expr = Node::new_ands(&[
        (Node::new_var("idx1", 0, 511) * 4 + Node::new_var("FLOAT4_INDEX", 0, 3)).lt(512),
        (Node::new_var("idx2", 0, 511) * 4 + Node::new_var("FLOAT8_INDEX", 0, 7)).lt(512),
    ]);
    test_variable(
        expr.floordiv(4, None),
        0,
        1,
        "((idx1<128) and ((idx2+(FLOAT8_INDEX//4))<128))",
    );
}

#[test]
fn test_lt_factors() {
    let expr = Node::new_ands(&[(Node::new_var("idx1", 0, 511) * 4
        + Node::new_var("FLOAT4_INDEX", 0, 256))
    .lt(512)]);
    test_variable(expr, 0, 1, "(((idx1*4)+FLOAT4_INDEX)<512)");
}

#[test]
fn test_div_becomes_num() {
    assert!(matches!(
        Node::new_var("a", 2, 3).floordiv(2, None),
        Node::Num { .. }
    ));
}

#[test]
fn test_var_becomes_num() {
    assert!(matches!(Node::new_var("a", 2, 2), Node::Num { .. }));
}

#[test]
fn test_equality() {
    let idx1 = Node::new_var("idx1", 0, 3);
    let idx2 = Node::new_var("idx2", 0, 3);

    assert_eq!(idx1.clone(), idx1.clone());
    assert_ne!(idx1.clone(), idx2.clone());
    assert_eq!(idx1.clone() * 4, idx1.clone() * 4);
    assert_ne!(idx1.clone() * 4, idx1.clone() * 3);
    assert_ne!(idx1.clone() * 4, idx1.clone() + 4);
    assert_ne!(idx1.clone() * 4, idx2.clone() * 4);
    assert_eq!(idx1.clone() + idx2.clone(), idx1.clone() + idx2.clone());
    assert_ne!(idx1.clone() + idx2.clone(), idx2.clone());
}

#[test]
fn test_factorize() {
    let a = Node::new_var("a", 0, 8);
    test_variable(a.clone() * 2 + a.clone() * 3, 0, 8 * 5, "(a*5)");
}

#[test]
fn test_factorize_no_mul() {
    let a = Node::new_var("a", 0, 8);
    test_variable(a.clone() + a.clone() * 3, 0, 8 * 4, "(a*4)");
}

#[test]
fn test_neg() {
    test_variable(-Node::new_var("a", 0, 8), -8, 0, "(a*-1)");
}

#[test]
fn test_add_1() {
    test_variable(Node::new_var("a", 0, 8) + 1, 1, 9, "(a+1)");
}

#[test]
fn test_add_num_1() {
    test_variable(Node::new_var("a", 0, 8) + Node::new_num(1), 1, 9, "(a+1)");
}

#[test]
fn test_sub_1() {
    test_variable(Node::new_var("a", 0, 8) - 1, -1, 7, "(a+-1)");
}

#[test]
fn test_sub_num_1() {
    test_variable(
        Node::new_var("a", 0, 8) - Node::new_num(1),
        -1,
        7,
        "(a+(1*-1))",
    );
}

#[test]
#[allow(clippy::erasing_op)]
fn test_mul_0() {
    test_variable(Node::new_var("a", 0, 8) * 0, 0, 0, "0");
}

#[test]
fn test_mul_1() {
    test_variable(Node::new_var("a", 0, 8) * 1, 0, 8, "a");
}

#[test]
fn test_mul_neg_1() {
    test_variable(
        (Node::new_var("a", 0, 2) * -1).floordiv(3, None),
        -1,
        0,
        "((((a*-1)+3)//3)+-1)",
    );
}

#[test]
fn test_mul_2() {
    test_variable(Node::new_var("a", 0, 8) * 2, 0, 16, "(a*2)");
}

#[test]
fn test_div_1() {
    test_variable(Node::new_var("a", 0, 8).floordiv(1, None), 0, 8, "a");
}

#[test]
fn test_mod_1() {
    test_variable(Node::new_var("a", 0, 8).modulus(1), 0, 0, "0");
}

#[test]
fn test_add_min_max() {
    test_variable(Node::new_var("a", 0, 8) * 2 + 12, 12, 16 + 12, "((a*2)+12)");
}

#[test]
fn test_div_min_max() {
    test_variable(Node::new_var("a", 0, 7).floordiv(2, None), 0, 3, "(a//2)");
}

#[test]
fn test_div_neg_min_max() {
    test_variable(
        Node::new_var("a", 0, 7).floordiv(-2, None),
        -3,
        0,
        "((a//2)*-1)",
    );
}

#[test]
fn test_sum_div_min_max() {
    test_variable(
        Node::new_sum(&[Node::new_var("a", 0, 7), Node::new_var("b", 0, 3)]).floordiv(2, None),
        0,
        5,
        "((a+b)//2)",
    );
}

#[test]
fn test_sum_div_factor() {
    test_variable(
        Node::new_sum(&[Node::new_var("a", 0, 7) * 4, Node::new_var("b", 0, 3) * 4])
            .floordiv(2, None),
        0,
        20,
        "((a*2)+(b*2))",
    );
}

#[test]
fn test_sum_div_some_factor() {
    test_variable(
        Node::new_sum(&[Node::new_var("a", 0, 7) * 5, Node::new_var("b", 0, 3) * 4])
            .floordiv(2, None),
        0,
        23,
        "((b*2)+((a*5)//2))",
    );
}

#[test]
fn test_sum_div_some_partial_factor() {
    test_variable(
        Node::new_sum(&[
            Node::new_num(16),
            Node::new_var("a", 0, 7) * 6,
            Node::new_var("b", 0, 7) * 6,
        ])
        .floordiv(16, None),
        1,
        6,
        "((((a*3)+(b*3))//8)+1)",
    );
}

#[test]
fn test_sum_div_no_factor() {
    test_variable(
        Node::new_sum(&[Node::new_var("a", 0, 7) * 5, Node::new_var("b", 0, 3) * 5])
            .floordiv(2, None),
        0,
        25,
        "(((a*5)+(b*5))//2)",
    );
}

#[test]
fn test_mod_factor() {
    test_variable(
        Node::new_sum(&[
            Node::new_var("a", 0, 7) * 100,
            Node::new_var("b", 0, 3) * 50,
        ])
        .modulus(100),
        0,
        99,
        "((b*50)%100)",
    );
}

#[test]
fn test_sum_div_const() {
    test_variable(
        Node::new_sum(&[Node::new_var("a", 0, 7) * 4, Node::new_num(3)]).floordiv(4, None),
        0,
        7,
        "a",
    );
}

#[test]
fn test_sum_div_const_big() {
    test_variable(
        Node::new_sum(&[Node::new_var("a", 0, 7) * 4, Node::new_num(3)]).floordiv(16, None),
        0,
        1,
        "(a//4)",
    );
}

#[test]
fn test_mod_mul() {
    test_variable((Node::new_var("a", 0, 5) * 10).modulus(9), 0, 5, "a");
}

#[test]
fn test_mul_mul() {
    test_variable((Node::new_var("a", 0, 5) * 10) * 9, 0, 5 * 10 * 9, "(a*90)");
}

#[test]
fn test_div_div() {
    test_variable(
        (Node::new_var("a", 0, 1800).floordiv(10, None)).floordiv(9, None),
        0,
        20,
        "(a//90)",
    );
}

#[test]
fn test_distribute_mul() {
    test_variable(
        Node::new_sum(&[Node::new_var("a", 0, 3), Node::new_var("b", 0, 5)]) * 3,
        0,
        24,
        "((a*3)+(b*3))",
    );
}

#[test]
fn test_mod_mul_sum() {
    test_variable(
        Node::new_sum(&[Node::new_var("b", 0, 2), Node::new_var("a", 0, 5) * 10]).modulus(9),
        0,
        7,
        "(a+b)",
    );
}

#[test]
fn test_sum_0() {
    test_variable(Node::new_sum(&[Node::new_var("a", 0, 7)]), 0, 7, "a");
}

#[test]
fn test_mod_remove() {
    test_variable(Node::new_var("a", 0, 6).modulus(100), 0, 6, "a");
}

#[test]
fn test_big_mod() {
    test_variable(Node::new_var("a", 0, 20).modulus(10), 0, 9, "(a%10)");
}

#[test]
fn test_gt_remove() {
    test_variable(Node::new_var("a", 0, 6).ge(25), 0, 0, "0");
}

#[test]
fn test_lt_remove() {
    test_variable(Node::new_var("a", 0, 6).lt(8), 1, 1, "1");
}

#[test]
fn test_and_fold() {
    test_variable(
        Node::new_ands(&[Node::new_num(0), Node::new_var("a", 0, 1)]),
        0,
        0,
        "0",
    );
}

#[test]
fn test_and_remove() {
    test_variable(
        Node::new_ands(&[Node::new_num(1), Node::new_var("a", 0, 1)]),
        0,
        1,
        "a",
    );
}

#[test]
fn test_mod_factor_negative() {
    test_variable(
        Node::new_sum(&[
            Node::new_num(-29),
            Node::new_var("a", 0, 100),
            Node::new_var("b", 0, 10) * 28,
        ])
        .modulus(28),
        0,
        27,
        "((a+27)%28)",
    );
}

#[test]
fn test_sum_combine_num() {
    test_variable(
        Node::new_sum(&[
            Node::new_num(29),
            Node::new_var("a", 0, 10),
            Node::new_num(-23),
        ]),
        6,
        16,
        "(a+6)",
    );
}

#[test]
fn test_sum_num_hoisted_and_factors_cancel_out() {
    test_variable(
        Node::new_sum(&[
            Node::new_var("a", 0, 1) * -4 + 1,
            Node::new_var("a", 0, 1) * 4,
        ]),
        1,
        1,
        "1",
    );
}

#[test]
fn test_div_factor() {
    test_variable(
        Node::new_sum(&[
            Node::new_num(-40),
            Node::new_var("a", 0, 10) * 2,
            Node::new_var("b", 0, 10) * 40,
        ])
        .floordiv(40, None),
        -1,
        9,
        "(b+-1)",
    );
}

#[test]
fn test_mul_div() {
    test_variable(
        (Node::new_var("a", 0, 10) * 4).floordiv(4, None),
        0,
        10,
        "a",
    );
}

#[test]
fn test_mul_div_factor_mul() {
    test_variable(
        (Node::new_var("a", 0, 10) * 8).floordiv(4, None),
        0,
        20,
        "(a*2)",
    );
}

#[test]
fn test_mul_div_factor_div() {
    test_variable(
        (Node::new_var("a", 0, 10) * 4).floordiv(8, None),
        0,
        5,
        "(a//2)",
    );
}

#[test]
fn test_div_remove() {
    test_variable(
        Node::new_sum(&[
            Node::new_var("idx0", 0, 127) * 4,
            Node::new_var("idx2", 0, 3),
        ])
        .floordiv(4, None),
        0,
        127,
        "idx0",
    );
}

#[test]
fn test_div_numerator_negative() {
    test_variable(
        (Node::new_var("idx", 0, 9) * -10).floordiv(11, None),
        -9,
        0,
        "((((idx*-10)+99)//11)+-9)",
    );
}

#[test]
fn test_div_into_mod() {
    test_variable(
        (Node::new_var("idx", 0, 16) * 4)
            .modulus(8)
            .floordiv(4, None),
        0,
        1,
        "(idx%2)",
    );
}

fn test_numeric(fx: fn(node: &dyn Node) -> &dyn Node, fi: fn(val: isize) -> isize) {
    for i in 0..10 {
        let x = fx(num(i));
        let (min, max) = x.min_max();
        assert_eq!(min, max);
        assert_eq!(min, fi(i));
    }

    for kmin in 0..10 {
        for kmax in 0..10 {
            if kmin > kmax {
                continue;
            }

            let v = fx(Node::new_var("tmp", kmin, kmax));
            let (min, max) = v.min_max();

            let values: Vec<isize> = (kmin..kmax + 1).map(&fi).collect();
            let min_value = values.iter().min().unwrap();
            let max_value = values.iter().max().unwrap();

            assert!(min <= *min_value);
            assert!(max >= *max_value);
        }
    }
}

#[test]
fn test_mod_4() {
    test_numeric(|x| x.modulus(4), |x| x % 4);
}

#[test]
fn test_div_4() {
    test_numeric(|x| x.floordiv(4, None), |x| x / 4);
}

#[test]
fn test_plus_1_div_2() {
    test_numeric(|x| (x + 1).floordiv(2, None), |x| (x + 1) / 2);
}

#[test]
fn test_plus_1_mod_2() {
    test_numeric(|x| (x + 1).modulus(2), |x| (x + 1) % 2);
}

#[test]
fn test_times_2() {
    test_numeric(|x| x * 2, |x| x * 2);
}

#[test]
fn test_times_2_plus_3() {
    test_numeric(|x| x * 2 + 3, |x| x * 2 + 3);
}

#[test]
fn test_times_2_plus_3_mod_4() {
    test_numeric(|x| (x * 2 + 3).modulus(4), |x| (x * 2 + 3) % 4);
}

#[test]
fn test_times_2_plus_3_div_4() {
    test_numeric(|x| (x * 2 + 3).modulus(4), |x| (x * 2 + 3) % 4);
}

#[test]
fn test_times_2_plus_3_div_4_mod_4() {
    test_numeric(
        |x| ((x * 2 + 3).floordiv(4, None)).modulus(4),
        |x| ((x * 2 + 3) / 4) % 4,
    );
}
