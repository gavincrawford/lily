use super::*;

#[test]
fn global_variables() {
    let (i, _) = interpret!("global_variables.ly");
    var_eq_literal!(i, "a", 1);
    var_eq_literal!(i, "b", true);
    var_eq_literal!(i, "c", "str");
    var_eq_literal!(i, "d", 'c');
}

#[test]
fn math() {
    let (i, _) = interpret!("math.ly");
    var_eq_literal!(i, "a", 1);
    var_eq_literal!(i, "b", 2.5);
    var_eq_literal!(i, "c", 6);
}

#[test]
fn operators() {
    let (i, _) = interpret!("operators.ly");
    var_eq_literal!(i, "n_eq", true);
    var_eq_literal!(i, "n_neq", true);
    var_eq_literal!(i, "n_add", 2);
    var_eq_literal!(i, "n_sub", 0);
    var_eq_literal!(i, "n_mul", 8);
    var_eq_literal!(i, "n_div", 8);
    var_eq_literal!(i, "b_eq", true);
    var_eq_literal!(i, "b_neq", true);
    var_eq_literal!(i, "s_eq", true);
    var_eq_literal!(i, "s_neq", true);
    var_eq_literal!(i, "s_add_s", "abcd");
    var_eq_literal!(i, "c_eq", true);
    var_eq_literal!(i, "c_neq", true);
}

#[test]
fn conditionals() {
    let (i, _) = interpret!("conditionals.ly");
    var_eq_literal!(i, "a", 8);
}

#[test]
fn functions() {
    let (i, _) = interpret!("functions.ly");
    var_eq_literal!(i, "a", 10);
    var_eq_literal!(i, "b", 20);
    var_eq_literal!(i, "c", true);
}

#[test]
fn loops() {
    let (i, _) = interpret!("loops.ly");
    var_eq_literal!(i, "i", 25);
    var_eq_literal!(i, "a", 25);
}

#[test]
fn indices() {
    let (i, _) = interpret!("indices.ly");
    var_eq_literal!(i, "idx_a", 2);
    var_eq_literal!(i, "idx_b", 3);
    var_eq!(i, "idx_list_whole", node!([lit!(123)]));
    var_eq_literal!(i, "idx_list_part", 123);
    var_eq_literal!(i, "assignment_flat", 1);
    var_eq_literal!(i, "assignment_nested", 1);
    var_eq_literal!(i, "head", '0');
    var_eq_literal!(i, "tail", '5');
}

#[test]
fn dangling_indices() {
    let (i, _) = interpret!("dangling_indices.ly");
    var_eq_literal!(i, "dangling", 10);
}

#[test]
fn imports() {
    let (i, _) = interpret!("imports.ly");
    var_eq_literal!(i, "get_res", 4);
    var_eq_literal!(i, "assign_res", "reassignment value");
    var_eq_literal!(i, "decl_res", "declaration value");
}

#[test]
fn nested_imports() {
    let (i, _) = interpret!("nested_imports.ly");
    var_eq_literal!(i, "res", 4);
}

#[test]
fn structs() {
    let (i, _) = interpret!("structs.ly");
    var_eq_literal!(i, "av", 444);
    var_eq_literal!(i, "bv", 0);
    var_eq_literal!(i, "declaration", true);
}
