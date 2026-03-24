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

# build a deep chain of nodes and traverse it repeatedly
let i = 0
while i < 200 do
  let tree = new Node(1, new Node(2, -1, new Node(3, -1, new Node(4, -1, -1))), -1)
  let head = tree.v
  let l_value = tree.left().v
  let l_r_value = tree.left().right().v
  let l_r_r_value = tree.left().right().right().v
  i = i + 1
end
