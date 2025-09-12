func nested n do
  if true do
    if true do
      if true do
        if true do
          return n
        end
      end
    end
  end
end

let nested_res = nested(1)

func chained n do
  return nested(n)
end

let chained_res = chained(2)
