let list = []

# test basic indexing
list = [1, 2, 3]
let idx = 2
let idx_a = list[4 - (1 * 3)]
let idx_b = list[idx]

# test nested indexes
list = [[123]]
let idx_list_whole = list[0]
let idx_list_part = list[0][0]

# test list assignment
list = [0, 0, 0, [0, 0]]
list[1] = 1
let assignment_flat = list[1]
list[3][0] = 1
let assignment_nested = list[3][0]

# test string indexing
let string = "012345"
let head = string[0]
let tail = string[5]
