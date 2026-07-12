struct Inner
  let val = 41

  func inc do
    val = val + 1
    return val
  end
end

struct Outer
  let inner = 0
end

# One-deep
let o1 = new Outer()
o1.inner = new Inner()
let inc1 = o1.inner.inc()
let val1 = o1.inner.val

# Two-deep
let o2 = new Outer()
o2.inner = o1
let inc2 = o2.inner.inner.inc()
let val2 = o2.inner.inner.val
