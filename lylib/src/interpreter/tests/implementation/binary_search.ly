func search list target do
  let left = 0
  # TODO length builtin
  # ... = len(list) - 1
  let right = 8
  while left <= right do
    let mid = (left + right) // 2
    if list[mid] == target do
      return mid
    else
      if list[mid] < target do
        left = mid + 1
      else
        right = mid - 1
      end
    end
  end
  return -1
end

let list = [8, 5, 7, 0, 2, 1, 3, 4, 6, 9]
let result = search(list, 2)
