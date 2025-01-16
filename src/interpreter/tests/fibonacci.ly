func fib n do
  let v = 0
  if n <= 1 do
    v = n
  else
    v = fib((n - 1)) + fib((n - 2))
  end
  return v
end
let result = fib(8)
