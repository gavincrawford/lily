struct DefinedConstructor do
  func constructor n do
    value = n
  end
  let value = 0
end

struct DefaultConstructor do
  let value = 0
end

# test constructors
let a = new DefinedConstructor(123)
let av = a.value
let b = new DefaultConstructor()
let bv = b.value

# test defining values in structs
b.other_value = true
let declaration = b.other_value
