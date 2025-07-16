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
    s_eq := true,
    s_neq := true,
    s_add_s := "abcd",
    c_eq := true,
    c_neq := true
));

test!(conditionals => (
    a := 8
));

test!(functions => (
    a := 10,
    b := 20,
    c := true
));

test!(loops => (
    i := 25,
    a := 25
));

test!(indices => (
    idx_a := 2,
    idx_b := 3,
    idx_list_whole == node!([lit!(123)]),
    idx_list_part := 123,
    assignment_flat := 1,
    assignment_nested := 1,
    head := '0',
    tail := '5'
));

test!(dangling_indices => (
    dangling := 10
));

test!(imports => (
    get_res := 4,
    assign_res := "reassignment value",
    decl_res := "declaration value"
));

test!(nested_imports => (
    res := 4
));

test!(structs => (
    av := 444,
    bv := 0,
    declaration := true
));

