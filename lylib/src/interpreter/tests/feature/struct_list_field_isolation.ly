struct Box
  let items = [1, 2, 3]
end

let a = new Box()
let b = new Box()
a.items[0] = 99

let a_val = a.items[0]
let b_val = b.items[0]
