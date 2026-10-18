# ast-grep: search by code shape

`ast-grep` parses code into a syntax tree and matches patterns against it, so it ignores comments and strings and can say "a call with this argument" or "an impl block for this trait". It complements `rg`, which stays the default: read [the search skill](../SKILL.md) first. Its main trap is that a pattern that does not fit returns an empty or short result instead of an error.

## The command

```bash
ast-grep run -l rust -p 'impl MyTrait for $T { $$$ }' --no-ignore vcs --json=stream . \
  | jq -r '"\(.file):\(.range.start.line+1): \(.text|split("\n")[0])"'
```

- The default prints every line of every matched node, often thousands of tokens for a handful of `impl` blocks. The `jq` trim keeps one line per match.
- Name the language every time: `-l rust`, `-l python`, `-l ts`, `-l tsx`. `-l ts` skips `.tsx` files.
- `--no-ignore vcs` stops it honoring `.gitignore`, matching the base command of the search skill.
- Call it `ast-grep`, never `sg`. On Debian and Arch, `sg` is shadow-utils' `newgrp`.
- There is no flag to limit results, and a broad pattern can print hundreds of KB. Pipe to `head -N`.

## Patterns that silently fail

These return an empty or short result, not an error:

- A trait with supertrait bounds: `trait X { $$$ }` does not match `trait X: Send + Sync { .. }`.
- A function with generics or a return annotation: `function f($$$A) { $$$B }` does not match `function f<T>(..): R { .. }`, and `def f($$$A): $$$B` does not match `def f(..) -> R:`.
- A path-qualified trait: `impl Trait for T` does not match `impl a::b::Trait for T`.
- Anything inside macro arguments: `assert_eq!(f(x), 1)` and `vec![Type { .. }]` are invisible, so calls and struct literals written there are missed.

## The second call that recovers a definition

A `kind` rule matches the declaration by node type and name, whatever its modifiers:

```bash
ast-grep scan --inline-rules $'id: q\nlanguage: rust\nrule:\n  kind: trait_item\n  has:\n    field: name\n    regex: \'^NAME$\'\n' --no-ignore vcs --json=stream .
```

Node kinds: `trait_item` (Rust trait), `function_declaration` (TS/JS, with `-l tsx` for TSX), `function_definition` (Python). For a path-qualified implementor, match any trait and constrain its name:

```bash
ast-grep scan --inline-rules $'id: q\nlanguage: rust\nrule:\n  pattern: impl $TR for $T { $$$ }\nconstraints:\n  TR:\n    regex: NAME$\n' --no-ignore vcs --json=stream .
```

## When to use it, and the cross-check

Use it for code shape: calls with a given argument, a construct nested inside another, a struct literal, a rewrite that must only touch real code. Skip it for names, strings, files, and anything written inside macro arguments. If a result looks short for what was asked, run the matching `rg` regex from the search skill and compare: trust the larger result until you know why they differ.
