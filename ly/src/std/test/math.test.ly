# test for failures in accessing constants
let pi = math.PI;
let e = math.E;
let tau = math.TAU;
let phi = math.PHI;

# test min
let min = math.min(3, 2)
assert(min == 2)

# test max
let max = math.max(3, 4)
assert(max == 4)

# test abs
let abs_pos = math.abs(1)
let abs_neg = math.abs(-1)
assert(abs_pos == abs_neg)
