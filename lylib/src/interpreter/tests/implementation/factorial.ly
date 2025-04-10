func factorial n do
  if n == 0 do
    return 1
  else
    return n * factorial(n - 1)
  end
end
let six_fac = factorial(6);
let one_fac = factorial(1);
let zero_fac = factorial(0);
