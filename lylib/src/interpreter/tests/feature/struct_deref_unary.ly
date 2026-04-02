# Negate a function-call deref result of an instance
struct Wrapper
  func Wrapper v do
    value = v
  end
  let value = 0
end

func get_wrapper do
  return new Wrapper(5)
end

let neg = -get_wrapper().value

# Logical not on a function-call deref result
struct Flag
  func Flag v do
    active = v
  end
  let active = false
end

func get_flag do
  return new Flag(true)
end

let flipped = !get_flag().active
