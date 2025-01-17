func fib n do
  if n <= 1 do
    return n
  else
    let one = fib(n - 1)
    let two = fib(n - 2)
    return one + two
  end
end
let result = fib(8)
