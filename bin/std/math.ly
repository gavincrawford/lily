# pi
let PI = 3.14159265

# euler's number
let E = 2.71828183

# tau
let TAU = 6.28318548

# golden ratio
let PHI = 1.61803401

# returns the greater of the two numbers
func max a b do
  if a > b do
    return a
  else
    return b
  end
end

# returns the lesser of the two numbers
func min a b do
  if a < b do
    return a
  else
    return b
  end
end

# returns the absolute value
func abs n do
  if n < 0 do
    return n * -1
  else
    return n
  end
end
