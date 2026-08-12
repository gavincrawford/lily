struct Node
  let l = -1
  let r = -1
  let v = 0
  let hl = false
  let hr = false

  func Node nv do
    v = nv
  end

  func set_left child do
    l = child
    hl = true
  end

  func set_right child do
    r = child
    hr = true
  end
end

# depth-first traversal
func dfs node do
  let result = [node.v]
  if node.hl do
    result = result + dfs(node.l)
  end
  if node.hr do
    result = result + dfs(node.r)
  end
  return result
end

#         1
#        / \
#       2   3
#      / \ / \
#     4  5 6  7
let n4 = new Node(4)
let n5 = new Node(5)
let n6 = new Node(6)
let n7 = new Node(7)

let n2 = new Node(2)
n2.set_left(n4)
n2.set_right(n5)

let n3 = new Node(3)
n3.set_left(n6)
n3.set_right(n7)

let tree = new Node(1)
tree.set_left(n2)
tree.set_right(n3)

let result = dfs(tree)
