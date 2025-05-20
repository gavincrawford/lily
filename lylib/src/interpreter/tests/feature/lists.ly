let list = [1, 2, 3, ['a', 'b', 'c']]

func dangling_test do
  let list = [10, 20]
  return list[0]
end

let idx = 2
let idx_a = dangling_test()
let idx_b = list[1]
let idx_c = list[idx]
let idx_list_whole = list[3]
let idx_list_part = list[3][0]
