func breaks_immediately do
  break
end

let i = 0
while i < 5 do
  breaks_immediately()
  i = i + 1
  print(i)
end
print("done")
