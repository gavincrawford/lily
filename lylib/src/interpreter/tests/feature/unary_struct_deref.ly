struct Counter
  func Counter n do
    val = n
  end
  let val = 0
end

let c = new Counter(5)
c.val++
let after_inc = c.val
c.val--
c.val--
let after_dec = c.val
let negate = -c.val
