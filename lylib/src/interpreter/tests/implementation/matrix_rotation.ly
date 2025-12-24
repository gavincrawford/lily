func rotate matrix do
  let new_matrix = [
    [matrix[0][0], matrix[1][0], matrix[2][0]],
    [matrix[0][1], matrix[1][1], matrix[2][1]],
    [matrix[0][2], matrix[1][2], matrix[2][2]],
  ]
  return new_matrix
end

let matrix_a = rotate([
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9],
])

let matrix_b = rotate([
  [1, 1, 1],
  [1, 1, 1],
  [1, 1, 1],
])
