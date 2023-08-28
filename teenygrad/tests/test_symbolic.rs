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

use teenygrad::shape::symbolic::{ands, num, sum, var, Node};

fn test_variable(v: Box<dyn Node>, min: isize, max: isize, s: &str) {
    let (vmin, vmax) = (v.min().unwrap(), v.max().unwrap());

    assert_eq!(format!("{}", v.render(false, false)), s);
    assert_eq!(vmin, min);
    assert_eq!(vmax, max);
}

#[test]
fn test_ge() {
    let node = var("a", 3, 8);

    test_variable(node.ge(num(77).as_ref()), 0, 0, "0");
    test_variable(node.ge(num(9).as_ref()), 0, 0, "0");
    test_variable(node.ge(num(8).as_ref()), 0, 1, "((a*-1)<-7)");
    test_variable(node.ge(num(4).as_ref()), 0, 1, "((a*-1)<-3)");
    test_variable(node.ge(num(3).as_ref()), 1, 1, "1");
    test_variable(node.ge(num(2).as_ref()), 1, 1, "1");
}

#[test]
fn test_lt() {
    let node = var("a", 3, 8);

    test_variable(node.lt(num(77).as_ref()), 1, 1, "1");
    test_variable(node.lt(num(9).as_ref()), 1, 1, "1");
    test_variable(node.lt(num(8).as_ref()), 0, 1, "(a<8)");
    test_variable(node.lt(num(4).as_ref()), 0, 1, "(a<4)");
    test_variable(node.lt(num(3).as_ref()), 0, 0, "0");
    test_variable(node.lt(num(2).as_ref()), 0, 0, "0");
}

#[test]
fn test_ge_divides() {
    let expr = (var("idx", 0, 511).mul(num(4).as_ref()))
        .add(var("FLOAT4_INDEX", 0, 3).as_ref())
        .lt(num(512).as_ref());

    test_variable(expr.clone(), 0, 1, "((idx*4)<512)");
    test_variable(
        expr.floordiv(num(4).as_ref(), Some(false)),
        0,
        1,
        "(idx<128)",
    );
}

#[test]
fn test_ge_divides_and() {
    let a = var("idx1", 0, 511)
        .mul(num(4).as_ref())
        .add(var("FLOAT4_INDEX", 0, 3).as_ref());
    let b = var("idx2", 0, 511)
        .mul(num(4).as_ref())
        .add(var("FLOAT4_INDEX", 0, 3).as_ref())
        .lt(num(512).as_ref());
    let expr = ands(&[a.as_ref(), b.as_ref()]);
    println!("{}", expr.render(true, false));
    let x = expr.floordiv(num(4).as_ref(), None);

    test_variable(x, 0, 1, "((idx1<128) and (idx2<128))");

    let a = var("idx1", 0, 511)
        .mul(num(4).as_ref())
        .add(var("FLOAT4_INDEX", 0, 3).as_ref())
        .lt(num(512).as_ref());
    let b = var("idx2", 0, 511)
        .mul(num(4).as_ref())
        .add(var("FLOAT8_INDEX", 0, 7).as_ref())
        .lt(num(512).as_ref());
    let expr = ands(&[a.as_ref(), b.as_ref()]);

    test_variable(
        expr.floordiv(num(4).as_ref(), None),
        0,
        1,
        "((idx1<128) and ((idx2+(FLOAT8_INDEX//4))<128))",
    );
}

#[test]
fn test_lt_factors() {
    let expr = ands(&[var("idx1", 0, 511)
        .mul(num(4).as_ref())
        .add(var("FLOAT4_INDEX", 0, 256).as_ref())
        .lt(num(512).as_ref())
        .as_ref()]);
    test_variable(expr, 0, 1, "(((idx1*4)+FLOAT4_INDEX)<512)");
}

#[test]
fn test_div_becomes_num() {
    assert!(var("a", 2, 3).floordiv(num(2).as_ref(), None).is_num());
}

#[test]
fn test_var_becomes_num() {
    assert!(var("a", 2, 2).is_num());
}

#[test]
fn test_equality() {
    // let idx1 = var("idx1", 0, 3);
    // let idx2 = var("idx2", 0, 3);

    // TODO - fix test cases
    // assert_eq!(idx1, idx1);
    // assert_ne!(idx1, idx2);
    // assert_eq!(&idx1 * 4, idx1.clone() * 4);
    // assert_ne!(&idx1 * 4, idx1.clone() * 3);
    // assert_ne!(&idx1 * 4, idx1.clone() + 4);
    // assert_ne!(&idx1 * 4, idx2.clone() * 4);
    // assert_eq!(&idx1 + idx2.clone(), idx1.clone() + idx2.clone());
    // assert_ne!(&idx1 + idx2.clone(), idx2.clone());
}

#[test]
fn test_factorize() {
    let a = var("a", 0, 8);
    test_variable(
        a.mul(num(2).as_ref()).add(a.mul(num(3).as_ref()).as_ref()),
        0,
        8 * 5,
        "(a*5)",
    );
}

#[test]
fn test_factorize_no_mul() {
    let a = var("a", 0, 8);
    test_variable(a.add(a.mul(num(3).as_ref()).as_ref()), 0, 8 * 4, "(a*4)");
}

#[test]
fn test_neg() {
    test_variable(var("a", 0, 8).neg(), -8, 0, "(a*-1)");
}

#[test]
fn test_add_1() {
    test_variable(var("a", 0, 8).add(num(1).as_ref()), 1, 9, "(a+1)");
}

#[test]
fn test_add_num_1() {
    test_variable(var("a", 0, 8).add(num(1).as_ref()), 1, 9, "(a+1)");
}

#[test]
fn test_sub_1() {
    test_variable(var("a", 0, 8).sub(num(1).as_ref()), -1, 7, "(a+-1)");
}

#[test]
fn test_sub_num_1() {
    test_variable(var("a", 0, 8).sub(num(1).as_ref()), -1, 7, "(a+(1*-1))");
}

#[test]
#[allow(clippy::erasing_op)]
fn test_mul_0() {
    test_variable(var("a", 0, 8).mul(num(0).as_ref()), 0, 0, "0");
}

#[test]
fn test_mul_1() {
    test_variable(var("a", 0, 8).mul(num(1).as_ref()), 0, 8, "a");
}

#[test]
fn test_mul_neg_1() {
    test_variable(
        var("a", 0, 2)
            .mul(num(-1).as_ref())
            .floordiv(num(3).as_ref(), None),
        -1,
        0,
        "((((a*-1)+3)//3)+-1)",
    );
}

#[test]
fn test_mul_2() {
    test_variable(var("a", 0, 8).mul(num(2).as_ref()), 0, 16, "(a*2)");
}

#[test]
fn test_div_1() {
    test_variable(var("a", 0, 8).floordiv(num(1).as_ref(), None), 0, 8, "a");
}

#[test]
fn test_mod_1() {
    test_variable(var("a", 0, 8).modulus(num(1).as_ref()), 0, 0, "0");
}

#[test]
fn test_add_min_max() {
    test_variable(
        var("a", 0, 8).mul(num(2).as_ref()).add(num(12).as_ref()),
        12,
        16 + 12,
        "((a*2)+12)",
    );
}

#[test]
fn test_div_min_max() {
    test_variable(
        var("a", 0, 7).floordiv(num(2).as_ref(), None),
        0,
        3,
        "(a//2)",
    );
}

#[test]
fn test_div_neg_min_max() {
    test_variable(
        var("a", 0, 7).floordiv(num(-2).as_ref(), None),
        -3,
        0,
        "((a//2)*-1)",
    );
}

#[test]
fn test_sum_div_min_max() {
    test_variable(
        sum(&[var("a", 0, 7).as_ref(), var("b", 0, 3).as_ref()]).floordiv(num(2).as_ref(), None),
        0,
        5,
        "((a+b)//2)",
    );
}

#[test]
fn test_sum_div_factor() {
    test_variable(
        sum(&[
            var("a", 0, 7).mul(num(4).as_ref()).as_ref(),
            var("b", 0, 3).mul(num(4).as_ref()).as_ref(),
        ])
        .floordiv(num(2).as_ref(), None),
        0,
        20,
        "((a*2)+(b*2))",
    );
}

#[test]
fn test_sum_div_some_factor() {
    test_variable(
        sum(&[
            var("a", 0, 7).mul(num(5).as_ref()).as_ref(),
            var("b", 0, 3).mul(num(4).as_ref()).as_ref(),
        ])
        .floordiv(num(2).as_ref(), None),
        0,
        23,
        "((b*2)+((a*5)//2))",
    );
}

#[test]
fn test_sum_div_some_partial_factor() {
    test_variable(
        sum(&[
            num(16).as_ref(),
            var("a", 0, 7).mul(num(6).as_ref()).as_ref(),
            var("b", 0, 7).mul(num(6).as_ref()).as_ref(),
        ])
        .floordiv(num(16).as_ref(), None),
        1,
        6,
        "((((a*3)+(b*3))//8)+1)",
    );
}

#[test]
fn test_sum_div_no_factor() {
    test_variable(
        sum(&[
            var("a", 0, 7).mul(num(5).as_ref()).as_ref(),
            var("b", 0, 3).mul(num(5).as_ref()).as_ref(),
        ])
        .floordiv(num(2).as_ref(), None),
        0,
        25,
        "(((a*5)+(b*5))//2)",
    );
}

#[test]
fn test_mod_factor() {
    test_variable(
        sum(&[
            var("a", 0, 7).mul(num(100).as_ref()).as_ref(),
            var("b", 0, 3).mul(num(50).as_ref()).as_ref(),
        ])
        .modulus(num(100).as_ref()),
        0,
        99,
        "((b*50)%100)",
    );
}

#[test]
fn test_sum_div_const() {
    test_variable(
        sum(&[
            var("a", 0, 7).mul(num(4).as_ref()).as_ref(),
            num(3).as_ref(),
        ])
        .floordiv(num(4).as_ref(), None),
        0,
        7,
        "a",
    );
}

#[test]
fn test_sum_div_const_big() {
    test_variable(
        sum(&[
            var("a", 0, 7).mul(num(4).as_ref()).as_ref(),
            num(3).as_ref(),
        ])
        .floordiv(num(16).as_ref(), None),
        0,
        1,
        "(a//4)",
    );
}

#[test]
fn test_mod_mul() {
    test_variable(
        var("a", 0, 5)
            .mul(num(10).as_ref())
            .modulus(num(9).as_ref()),
        0,
        5,
        "a",
    );
}

#[test]
fn test_mul_mul() {
    test_variable(
        var("a", 0, 5).mul(num(10).as_ref()).mul(num(9).as_ref()),
        0,
        5 * 10 * 9,
        "(a*90)",
    );
}

#[test]
fn test_div_div() {
    test_variable(
        var("a", 0, 1800)
            .floordiv(num(10).as_ref(), None)
            .floordiv(num(9).as_ref(), None),
        0,
        20,
        "(a//90)",
    );
}

#[test]
fn test_distribute_mul() {
    test_variable(
        sum(&[var("a", 0, 3).as_ref(), var("b", 0, 5).as_ref()]).mul(num(3).as_ref()),
        0,
        24,
        "((a*3)+(b*3))",
    );
}

#[test]
fn test_mod_mul_sum() {
    test_variable(
        sum(&[
            var("b", 0, 2).as_ref(),
            var("a", 0, 5).mul(num(10).as_ref()).as_ref(),
        ])
        .modulus(num(9).as_ref()),
        0,
        7,
        "(a+b)",
    );
}

#[test]
fn test_sum_0() {
    test_variable(sum(&[var("a", 0, 7).as_ref()]), 0, 7, "a");
}

#[test]
fn test_mod_remove() {
    test_variable(var("a", 0, 6).modulus(num(100).as_ref()), 0, 6, "a");
}

#[test]
fn test_big_mod() {
    test_variable(var("a", 0, 20).modulus(num(10).as_ref()), 0, 9, "(a%10)");
}

#[test]
fn test_gt_remove() {
    test_variable(var("a", 0, 6).ge(num(25).as_ref()), 0, 0, "0");
}

#[test]
fn test_lt_remove() {
    test_variable(var("a", 0, 6).lt(num(8).as_ref()), 1, 1, "1");
}

#[test]
fn test_and_fold() {
    test_variable(ands(&[num(0).as_ref(), var("a", 0, 1).as_ref()]), 0, 0, "0");
}

#[test]
fn test_and_remove() {
    test_variable(ands(&[num(1).as_ref(), var("a", 0, 1).as_ref()]), 0, 1, "a");
}

#[test]
fn test_mod_factor_negative() {
    test_variable(
        sum(&[
            num(-29).as_ref(),
            var("a", 0, 100).as_ref(),
            var("b", 0, 10).mul(num(28).as_ref()).as_ref(),
        ])
        .modulus(num(28).as_ref()),
        0,
        27,
        "((a+27)%28)",
    );
}

#[test]
fn test_sum_combine_num() {
    test_variable(
        sum(&[
            num(29).as_ref(),
            var("a", 0, 10).as_ref(),
            num(-23).as_ref(),
        ]),
        6,
        16,
        "(a+6)",
    );
}

#[test]
fn test_sum_num_hoisted_and_factors_cancel_out() {
    test_variable(
        sum(&[
            var("a", 0, 1)
                .mul(num(-4).as_ref())
                .add(num(1).as_ref())
                .as_ref(),
            var("a", 0, 1).mul(num(4).as_ref()).as_ref(),
        ]),
        1,
        1,
        "1",
    );
}

#[test]
fn test_div_factor() {
    test_variable(
        sum(&[
            num(-40).as_ref(),
            var("a", 0, 10).mul(num(2).as_ref()).as_ref(),
            var("b", 0, 10).mul(num(40).as_ref()).as_ref(),
        ])
        .floordiv(num(40).as_ref(), None),
        -1,
        9,
        "(b+-1)",
    );
}

#[test]
fn test_mul_div() {
    test_variable(
        var("a", 0, 10)
            .mul(num(4).as_ref())
            .floordiv(num(4).as_ref(), None),
        0,
        10,
        "a",
    );
}

#[test]
fn test_mul_div_factor_mul() {
    test_variable(
        var("a", 0, 10)
            .mul(num(8).as_ref())
            .floordiv(num(4).as_ref(), None),
        0,
        20,
        "(a*2)",
    );
}

#[test]
fn test_mul_div_factor_div() {
    test_variable(
        var("a", 0, 10)
            .mul(num(4).as_ref())
            .floordiv(num(8).as_ref(), None),
        0,
        5,
        "(a//2)",
    );
}

#[test]
fn test_div_remove() {
    test_variable(
        sum(&[
            var("idx0", 0, 127).mul(num(4).as_ref()).as_ref(),
            var("idx2", 0, 3).as_ref(),
        ])
        .floordiv(num(4).as_ref(), None),
        0,
        127,
        "idx0",
    );
}

#[test]
fn test_div_numerator_negative() {
    test_variable(
        var("idx", 0, 9)
            .mul(num(-10).as_ref())
            .floordiv(num(11).as_ref(), None),
        -9,
        0,
        "((((idx*-10)+99)//11)+-9)",
    );
}

#[test]
fn test_div_into_mod() {
    test_variable(
        var("idx", 0, 16)
            .mul(num(4).as_ref())
            .modulus(num(8).as_ref())
            .floordiv(num(4).as_ref(), None),
        0,
        1,
        "(idx%2)",
    );
}

fn test_numeric(fx: fn(node: Box<dyn Node>) -> Box<dyn Node>, fi: fn(val: isize) -> isize) {
    for i in 0..10 {
        let x = fx(num(i));
        let (min, max) = (x.min().unwrap(), x.max().unwrap());
        assert_eq!(min, max);
        assert_eq!(min, fi(i));
    }

    for kmin in 0..10 {
        for kmax in 0..10 {
            if kmin > kmax {
                continue;
            }

            let v = fx(var("tmp", kmin, kmax));
            let (min, max) = (v.min().unwrap(), v.max().unwrap());

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
    test_numeric(|x| x.modulus(num(4).as_ref()), |x| x % 4);
}

#[test]
fn test_div_4() {
    test_numeric(|x| x.floordiv(num(4).as_ref(), None), |x| x / 4);
}

#[test]
fn test_plus_1_div_2() {
    test_numeric(
        |x| (x.add(num(1).as_ref())).floordiv(num(2).as_ref(), None),
        |x| (x + 1) / 2,
    );
}

#[test]
fn test_plus_1_mod_2() {
    test_numeric(
        |x| (x.add(num(1).as_ref())).modulus(num(2).as_ref()),
        |x| (x + 1) % 2,
    );
}

#[test]
fn test_times_2() {
    test_numeric(|x| x.mul(num(2).as_ref()), |x| x * 2);
}

#[test]
fn test_times_2_plus_3() {
    test_numeric(
        |x| x.mul(num(2).as_ref()).add(num(3).as_ref()),
        |x| x * 2 + 3,
    );
}

#[test]
fn test_times_2_plus_3_mod_4() {
    test_numeric(
        |x| (x.mul(num(2).as_ref()).add(num(3).as_ref())).modulus(num(4).as_ref()),
        |x| (x * 2 + 3) % 4,
    );
}

#[test]
fn test_times_2_plus_3_div_4() {
    test_numeric(
        |x| (x.mul(num(2).as_ref()).add(num(3).as_ref())).modulus(num(4).as_ref()),
        |x| (x * 2 + 3) % 4,
    );
}

#[test]
fn test_times_2_plus_3_div_4_mod_4() {
    test_numeric(
        |x| {
            (x.mul(num(2).as_ref())
                .add(num(3).as_ref())
                .floordiv(num(4).as_ref(), None))
            .modulus(num(4).as_ref())
        },
        |x| ((x * 2 + 3) / 4) % 4,
    );
}
