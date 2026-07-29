let global_val = 10

struct Foo
  let value = 0
  func Foo do
    value = global_val
  end
end

let f = new Foo()
let result = f.value
