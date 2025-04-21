struct Number do
  func constructor n do
    value = n
  end
  let value = 0
end

let instance = new Number(123)
let value = instance.value
