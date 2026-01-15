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
let x = 10

if x > 5 do
  x = 1
end

if x == 1 do
  x = 2
else
  x = 3
end
```

### Loops
```lily
let i = 0
while i < 3 do
  i = i + 1
end

while true do
  break       # exit loop early
end
```

## Functions

```lily
func add a b do
  return a + b
end

# call
let result = add(2, 3)
```

## Structs

```lily
struct Point
  let x = 0
  let y = 0

  # constructor (same name as struct)
  func Point a b do
    x = a
    y = b
  end
end

let p = new Point(10, 20)
let px = p.x
p.y = 30
p.z = 40    # can add fields dynamically
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

```lily !skip
import "./math.ly" as math

let result = math.add(1, 2)
let value = math.pi
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
