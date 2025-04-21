struct DefinedConstructor do
  func constructor n do
    value = n
  end
  let value = 0
end

struct DefaultConstructor do
  let value = 0
end

let a = new DefinedConstructor(123)
let av = a.value
let b = new DefaultConstructor()
let bv = b.value
