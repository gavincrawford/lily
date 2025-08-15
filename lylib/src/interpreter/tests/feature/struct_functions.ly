struct Calculator
  let last = -1

  func add l r do
    let res = l + r
    last = res
    return res
  end

  func sub l r do
    let res = l - r
    last = res
    return res
  end

  func get_last do
    return last
  end
end

let calculator = new Calculator()
let add = calculator.add(5, 5)
let sub = calculator.sub(5, 5)
let last = calculator.get_last()
