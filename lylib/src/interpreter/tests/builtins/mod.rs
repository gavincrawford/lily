use super::*;

#[test]
fn chars() {
    let (i, _) = interpret!("chars.ly");
    var_eq!(
        i,
        "letters",
        node!([
            lit!('a'),
            lit!('b'),
            lit!('c'),
            lit!('1'),
            lit!('2'),
            lit!('3')
        ])
    );
}

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
