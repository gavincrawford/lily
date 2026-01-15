# Lily Language Syntax Reference

Quick reference for the Lily programming language (`.ly` files).

## Comments

```lily
# single-line comment
## also valid
```

## Data Types

| Type | Example | Notes |
|------|---------|-------|
| Number | `42`, `3.14`, `-5` | 32-bit float internally |
| String | `"hello"` | Double quotes |
| Char | `'c'` | Single quotes |
| Boolean | `true`, `false` | |
| List | `[1, 2, 3]` | Can be nested, mixed types |
| Undefined | `undefined` | |

## Variables

```lily
let x = 42          # declaration
x = 100             # assignment
```

## Operators

### Arithmetic
| Op | Description | Example |
|----|-------------|---------|
| `+` | Add / concat | `1 + 2`, `"a" + "b"`, `[1] + [2]` |
| `-` | Subtract | `5 - 3` |
| `*` | Multiply | `4 * 2` |
| `/` | Divide | `10 / 2` |
| `^` | Power | `2 ^ 3` |
| `//` | Floor divide | `7 // 2` → `3` |
| `++` | Increment | `x++` |
| `--` | Decrement | `x--` |
| `-` | Negate | `-x` |

### Comparison
| Op | Description |
|----|-------------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `<=` | Less or equal |
| `>` | Greater than |
| `>=` | Greater or equal |

### Logical
| Op | Description |
|----|-------------|
| `&&` | And |
| `\|\|` | Or |
| `!` | Not |

## Control Flow

### Conditionals
```lily
if condition do
  # body
end

if condition do
  # if body
else
  # else body
end
```

### Loops
```lily
while condition do
  # body
end

while true do
  break       # exit loop early
end
```

## Functions

```lily
func name param1 param2 do
  return value
end

# call
let result = name(arg1, arg2)
```

## Structs

```lily
struct Name
  let field = default_value

  # constructor (same name as struct)
  func Name arg do
    field = arg
  end
end

let instance = new Name(value)
let v = instance.field
instance.field = new_value
instance.new_field = value    # can add fields dynamically
```

## Lists

```lily
let list = [1, 2, 3]
let nested = [[1, 2], [3, 4]]
let empty = []

# indexing (0-based)
let first = list[0]
list[1] = 999

# concatenation
let combined = [1, 2] + [3, 4]
```

## Modules

```lily
import "./path/to/file.ly" as alias

let result = alias.function(args)
let value = alias.variable
```

## Built-in Functions

| Function | Description |
|----------|-------------|
| `print(value)` | Output to stdout |
| `len(list_or_string)` | Get length |
| `sort(list)` | Sort list (numbers or strings, not mixed) |
| `chars(string)` | Convert string to char list |
| `assert(condition)` | Error if false |

## String Operations

```lily
let concat = "hello" + " world"
let with_num = "count: " + 42
let with_char = "ab" + 'c'
let length = len("hello")
let char_list = chars("abc")    # ['a', 'b', 'c']
let indexed = "hello"[0]        # 'h'
```

## Truthiness

Truthy: `true`, non-zero numbers, non-empty strings/chars, lists, structs, functions
Falsy: `false`, `0`, `undefined`
