use super::*;

#[test]
fn len() {
    let (_, out) = interpret!("len.ly");
    assert_eq!(out, "0\n5\n");
}

#[test]
fn print() {
    let (_, out) = interpret!("print.ly");
    assert_eq!(out, "str\nc\n1\ntrue\n");
}
