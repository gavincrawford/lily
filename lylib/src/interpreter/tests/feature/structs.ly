struct DefinedConstructor do
  func constructor x y do
    a = x
    b = y
  end
  let a = 0
  let b = 0
end

struct DefaultConstructor do
  let value = 0
end

# test defined constructors with two values
let a = new DefinedConstructor(123, 321)
let av = a.a + a.b

# test default constructors with one value
let b = new DefaultConstructor()
let bv = b.value

# test defining values in structs
b.other_value = true
let declaration = b.other_value
