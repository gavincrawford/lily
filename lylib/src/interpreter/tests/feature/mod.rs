use super::*;

test!(global_variables => (
    a := 1,
    b := true,
    c := "str",
    d := 'c'
));

test!(math => (
    a := 1,
    b := 2.5,
    c := 6
));

test!(operators => (
    n_eq := true,
    n_neq := true,
    n_add := 2,
    n_sub := 0,
    n_mul := 8,
    n_div := 8,
    b_eq := true,
    b_neq := true,
    b_not := false,
    s_eq := true,
    s_neq := true,
    c_eq := true,
    c_neq := true
));

test!(ifelse => (
    a := 0
));

test!(comparison_numerical => (
    a := 8
));

test!(comparison_boolean => (
    a := 4
));

test!(functions => (
    a := 10,
    b := 20,
    c := true
));

test!(loops => (
    x := 25
));

test!(loops_nested => (
    x := 25
));

test!(indices => (
    idx_a := 2,
    idx_b := 3,
    assignment_flat := 1,
    assignment_nested := 1,
));

test!(indices_dangling => (
    dangling := 10
));

test!(indices_nested => (
    nest_0 == node!([lit!(123)]),
    nest_1 := 123,
));

test!(imports => (
    get_res := 4,
    assign_res := "reassignment value",
    decl_res := "declaration value"
));

test!(string_index => (
    head := '0',
    tail := '5'
));

test!(string_concat => (
    concatenated := "abcd"
));

test!(nested_imports => (
    res := 4
));

test!(structs => (
    av := 444,
    bv := 0,
    declaration := true
));
