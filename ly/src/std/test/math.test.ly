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

# test trunc returns whole integer part
assert(math.trunc(1.01) == 1)
assert(math.trunc(1.99) == 1)

# test acos/asin
let half_pi = pi / 2
assert(math.abs(math.acos(0) - half_pi) < 0.001)
assert(math.abs(math.acos(1) - 0) < 0.001)
assert(math.abs(math.asin(0) - 0) < 0.001)
assert(math.abs(math.asin(1) - half_pi) < 0.001)

# test exp
assert(math.abs(math.exp(0) - 1) < 0.001)
assert(math.abs(math.exp(1) - e) < 0.001)

# test cosh/sinh
assert(math.abs(math.cosh(0) - 1) < 0.001)
assert(math.abs(math.sinh(0) - 0) < 0.001)
