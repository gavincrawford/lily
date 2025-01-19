func factorial n do
  if n == 0 do
    return 1
  else
    return n * factorial(n - 1)
  end
end
let result = factorial(6);
