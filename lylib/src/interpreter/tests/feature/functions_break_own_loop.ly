func breaks_own_loop do
  let j = 0
  while j < 3 do
    j = j + 1
    break
  end
  return j
end

let i = 0
let out = ""
while i < 5 do
  breaks_own_loop()
  i = i + 1
  out = out + i
end
print(out)
