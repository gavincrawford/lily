let csv = "a,b,c"
let by_comma = split(csv, ',')

let spaced = "hello world foo"
let by_space = split(spaced, " ")

let no_delim = "abc"
let no_match = split(no_delim, ',')

let multi_char = "one::two::three"
let by_double_colon = split(multi_char, "::")
