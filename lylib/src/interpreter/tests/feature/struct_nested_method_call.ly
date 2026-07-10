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

let o = new Outer()
o.inner = new Inner()

let first = o.inner.inc()
let second = o.inner.inc()
let val = o.inner.val
