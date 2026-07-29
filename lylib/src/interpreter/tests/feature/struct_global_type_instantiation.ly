struct Inner
  let value = 0
  func Inner v do
    value = v
  end
end

struct Outer
  let inner = 0
  func Outer do
    inner = new Inner(5)
  end
end

let o = new Outer()
let result = o.inner.value
