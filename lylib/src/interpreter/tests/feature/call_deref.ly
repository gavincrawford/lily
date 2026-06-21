# Tests deref on a function call result

struct Point
  func Point a b do
    x = a
    y = b
  end
  let x = 0
  let y = 0
end

func make_point do
  return new Point(10, 20)
end

let px = make_point().x
let py = make_point().y
