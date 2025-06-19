use super::*;

#[test]
fn binary_search() {
    let (i, _) = interpret!("binary_search.ly");
    var_eq_literal!(i, "some", 4);
    var_eq_literal!(i, "none", -1);
}

#[test]
fn fibonacci() {
    let (i, _) = interpret!("fibonacci.ly");
    var_eq_literal!(i, "result", 21);
}

#[test]
fn factorial() {
    let (i, _) = interpret!("factorial.ly");
    var_eq_literal!(i, "six_fac", 720);
    var_eq_literal!(i, "one_fac", 1);
    var_eq_literal!(i, "zero_fac", 1);
}

#[test]
fn matrix_rotation() {
    let (i, _) = interpret!("matrix_rotation.ly");
    var_eq!(
        i,
        "result",
        node!([
            node!([lit!(1), lit!(4), lit!(7)]),
            node!([lit!(2), lit!(5), lit!(8)]),
            node!([lit!(3), lit!(6), lit!(9)])
        ])
    );
}
