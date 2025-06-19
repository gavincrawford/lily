func search list target do
  let left = 0
  let right = len(list) - 1
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

let some = search([8, 5, 7, 0, 2, 1, 3, 4, 6, 9], 2)
let none = search([], 1)
