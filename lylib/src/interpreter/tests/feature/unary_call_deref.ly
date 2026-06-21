# Tests unary operators in the specific case:
# 1. Function returns instance
# 2. Deref applies to function return value
# 3. Unary applies to deref result

struct Wrapper
  func Wrapper i b do
    int = i
    bool = b
  end
  let int = 0
  let bool = false
end

func get_wrapper do
  return new Wrapper(5, true)
end

# Test negation
let neg = -get_wrapper().int

# Test logical not
let flipped = !get_wrapper().bool
