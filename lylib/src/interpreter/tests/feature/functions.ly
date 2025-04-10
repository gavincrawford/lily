func add l r do
  return l + r
end
let a = add(5, 5)

func double value do
  return add(value, value)
end
let b = double(10)

func greater_than lhs rhs do
  if lhs > rhs do
    return true
  end
  return false
end
let c = greater_than(2, 1)

