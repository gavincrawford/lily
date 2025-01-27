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
