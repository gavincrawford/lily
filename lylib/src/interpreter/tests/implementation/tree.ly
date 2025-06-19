# TODO using negative values for nonexistence causes problems in initialization...
# ex. new Node(1, new Node(2, -1, -1), -1)
struct Node do
  let l = 0
  let r = 0
  let v = 0
  func constructor nv nl nr do
    v = nv
    l = nl
    r = nr
  end
end

let tree = new Node(1, new Node(2, 0, 0), 0)

# check tree head == 1
let head = tree.v

# check left hand side value == 2
let lhs = tree.l.v

# check right hand side value doesn't exist
let rhs = tree.r
