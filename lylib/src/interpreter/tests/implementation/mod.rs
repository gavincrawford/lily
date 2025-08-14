use super::*;

test!(binary_search => (
    some := 4,
    none := -1
));

test!(tree => (
    head := 1,
    lhs := 2,
    rhs := -1
));

test!(fibonacci => (
    result := 21
));

test!(factorial => (
    six_fac := 720,
    one_fac := 1,
    zero_fac := 1
));

test!(matrix_rotation => (
    result == node!([
        node!([lit!(1), lit!(4), lit!(7)]),
        node!([lit!(2), lit!(5), lit!(8)]),
        node!([lit!(3), lit!(6), lit!(9)])
    ])
));
