func get_value do
  return 42
end

func get_value_nested do
  get_value()
  return "reached"
end

print(get_value_nested())
