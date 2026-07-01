import "./imports_src.ly" as src

# Test function calls across modules
let get_res = src.add(2, 2)

# Test reassignment
src.v = "reassignment value"
let assign_res = src.v

# Test declaring to a module
src.decl = "declaration value"
let decl_res = src.decl

# Test setting outer (see `imports_src.ly`)
src.set_outer()
let outer_res = src.outer

