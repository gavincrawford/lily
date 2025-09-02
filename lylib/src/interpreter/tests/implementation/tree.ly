struct Node
  let l = -1
  let r = -1
  let v = 0

  func Node nv nl nr do
    v = nv
    l = nl
    r = nr
  end

  func left do
    return l
  end

  func right do
    return r
  end
end

let tree = new Node(1, new Node(2, -1, -1), -1)

# check tree head == 1
let head = tree.v

# check left hand side value == 2
let l_value = tree.left().v

# check right hand side value doesn't exist
let r_does_not_exist = tree.right()
