# test numeric index assignment
let arr = [100, 200, 300];
let idx = 1;

# test literal index assignment
arr[0] = 999;

# test variable index assignment
arr[idx] = 888;

# test expression index assignment
arr[(2 - 1)] = 777;

let result = arr;
