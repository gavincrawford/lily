func dangling_test do
  let list = [10, 20]
  return list[0]
end
let dangling = dangling_test()
