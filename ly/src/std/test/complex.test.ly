let cmplx = new complex.Complex(0, 0)

# test add
cmplx = new complex.Complex(1, 2)
cmplx.add(new complex.Complex(3, 6))
assert(cmplx.re == 4 && cmplx.im == 8)

# test sub
cmplx = new complex.Complex(2, 2)
cmplx.sub(new complex.Complex(2, 4))
assert(cmplx.re == 0 && cmplx.im == -2)

# test mul
cmplx = new complex.Complex(1, 1)
cmplx.mul(new complex.Complex(5, 4))
assert(cmplx.re == 1 && cmplx.im == 9)

# test div
cmplx = new complex.Complex(1, 1)
cmplx.div(new complex.Complex(1, 2))
assert(cmplx.re == 0.6 && cmplx.im == -0.2)

# test magnitude
cmplx = new complex.Complex(3, 4)
assert(cmplx.mag() == 5)

# test string conversion
cmplx = new complex.Complex(3, 3)
assert("3 + 3i" == cmplx.as_string())
