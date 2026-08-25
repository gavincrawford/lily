# logical && and || operator results

# and truth table
let and_tt = true && true
let and_tf = true && false
let and_ft = false && true
let and_ff = false && false

# or truth table
let or_tt = true || true
let or_tf = true || false
let or_ft = false || true
let or_ff = false || false

# chaining
let and_chain = true && true && false
let or_chain = false || false || true

# mixed precedence -- && binds tighter than ||
let mixed = true || false && false
let mixed_b = false && false || true

# combined with negation
let neg_and = !false && true
let neg_or = !true || false
let neg_group = !(!true && !false)

# combined with comparisons
let cmp = (1 < 2) && (3 > 2)
let cmp_b = (1 > 2) || (2 == 2)
