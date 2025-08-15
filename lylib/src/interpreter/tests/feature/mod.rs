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
    b_not := true,
    s_eq := true,
    s_neq := true,
    c_eq := true,
    c_neq := true
));

test!(operators_precedence => (
    a := 8,
    b := true,
    c := true,
    d := true
));

test!(return_base_scope => panic);

test!(ifelse => (
    a := 0
));

test!(comparison_numerical => (
    a := 8
));

test!(comparison_boolean => (
    a := 6
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

test!(indices_dangling => (
    dangling := 10
));

test!(indices_indirect => (
    read := 5,
    write := 0
));

test!(indices_assignment => (
    result == node!([
        lit!(999),
        lit!(777),
        lit!(300)
    ])
));

test!(indices_nested => (
    nest_0 == node!([lit!(123)]),
    nest_1 := 123
));

test!(indices_out_of_range => panic);

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
    str_concat := "abcd",
    num_concat := "123",
    num_concat_reverse := "123",
    char_concat := "xyz",
    char_concat_reverse := "xyz"
));

test!(nested_imports => (
    res := 4
));

test!(struct_constructors => (
    av := 444,
    bv := 0,
    declaration := true
));

test!(struct_functions => (
    add := 10,
    sub := 0,
    last := 0
));
