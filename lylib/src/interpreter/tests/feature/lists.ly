let list = []

# test basic indexing
list = [1, 2, 3]
let idx = 2
let idx_a = list[4 - (1 * 3)]
let idx_b = list[idx]

# test dangling references
func dangling_test do
  let list = [10, 20]
  return list[0]
end
let dangling = dangling_test()

# test nested indexes
list = [[123]]
let idx_list_whole = list[0]
let idx_list_part = list[0][0]

# test list assignment
list = [0, 0, 0, 0]
list[1] = 1
let assignment = list[1]
