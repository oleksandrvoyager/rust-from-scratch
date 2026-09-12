# env-config format (reference)

```
# comment
NAME = value
HOST = localhost
URL = https://${HOST}:8080/api
```

- One assignment per line: `IDENTIFIER = VALUE`.
- Identifier: letters, digits, `_`; can't start with a digit.
- `=` separates name from value; surrounding whitespace is insignificant.
- Value: rest of the line (trimmed), may contain zero or more `${IDENTIFIER}`
  references mixed with literal text.
- Lines starting with `#` (after leading whitespace) are comments. Blank
  lines are ignored.
- References may point at names defined anywhere in the file (forward or
  backward), and a referenced value may itself contain further references.
