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

# returns the integer part of n
func trunc n do
  if n < 0 do
    return -((-n) // 1)
  else
    return n // 1
  end
end

# returns e^n
func exp n do
  return E ^ n
end

# inverse cos of n
func acos n do
  let lo = 0
  let hi = PI
  let i = 0
  while i < 30 do
    let mid = (lo + hi) / 2
    if cos(mid) > n do
      lo = mid
    else
      hi = mid
    end
    i++
  end
  return (lo + hi) / 2
end

# inverse sin of n
func asin n do
  let lo = -(PI / 2)
  let hi = PI / 2
  let i = 0
  while i < 30 do
    let mid = (lo + hi) / 2
    if sin(mid) < n do
      lo = mid
    else
      hi = mid
    end
    i++
  end
  return (lo + hi) / 2
end

# hyperbolic cos
func cosh n do
  return (exp(n) + exp(-n)) / 2
end

# hyperbolic sin
func sinh n do
  return (exp(n) - exp(-n)) / 2
end
