struct Box
  let size = 0

  func Box items do
    size = len(items)
  end

  func describe do
    return len("box")
  end
end

let b = new Box([1, 2, 3])
let result = b.size
let method_result = b.describe()
