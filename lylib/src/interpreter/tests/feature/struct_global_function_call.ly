func helper n do
  return n * 2
end

struct Foo
  let value = 0
  func Foo v do
    value = helper(v)
  end
end

let f = new Foo(5)
let result = f.value
