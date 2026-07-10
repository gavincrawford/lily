let n_add = 1
n_add += 4

let n_sub = 10
n_sub -= 3

let n_mul = 3
n_mul *= 4

let n_div = 20
n_div /= 4

let n_expr = 1
n_expr += 2 + 3

struct Point
  let x = 0

  func Point a do
    x = a
  end
end

let p = new Point(1)
p.x += 9
let n_deref = p.x
