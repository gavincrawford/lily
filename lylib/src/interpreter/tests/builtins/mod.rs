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

test!(len => (
    empty_list := 0,
    empty_string := 0,
    string := 11,
    list := 5,
    unicode := 4
));

test!(print => "str\nc\n1\ntrue\n\n");

test!(sort => (
    sorted_numbers == node!([lit!(1), lit!(2), lit!(3), lit!(4), lit!(5)]),
    sorted_words == node!([lit!("apple"), lit!("banana"), lit!("mango")]),
    empty == node!([]),
    single == node!([lit!(42)]),
    reverse_sorted == node!([lit!(1), lit!(2), lit!(3), lit!(4), lit!(5)]),
    duplicates == node!([lit!(1), lit!(1), lit!(2), lit!(2), lit!(3), lit!(3)]),
    negatives == node!([lit!(-3), lit!(-2), lit!(-1), lit!(0), lit!(1)])
));

test!(sort_mixed_types => panic);
