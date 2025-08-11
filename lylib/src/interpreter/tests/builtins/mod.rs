use super::*;

test!(assert_fails => panic);

test!(assert_passes => "");

test!(chars => (
    letters == node!([
        lit!('a'),
        lit!('b'),
        lit!('c'),
        lit!('1'),
        lit!('2'),
        lit!('3')
    ])
));

test!(len => "0\n5\n");

test!(print => "str\nc\n1\ntrue\n");

test!(sort => (
    sorted_list == node!([
        lit!(1),
        lit!(2),
        lit!(3),
        lit!(4),
        lit!(5)
    ])
));
