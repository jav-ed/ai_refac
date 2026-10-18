---
name: search
description: Finds definitions, implementors, callers, struct literals, exact strings, topics and file names inside local files and folders (code, docs, configs, logs) with ripgrep and ast-grep, getting the answer in one command with little noise. Use when about to search, grep or find something in a repository or folder, when asked where something is defined, used or mentioned, or when a search returns nothing or less than expected; not for web search.
---

# search

Use `rg` by default and `ast-grep` for code shape. What decides the result is the shape of the regex and the ignore flags, more than the tool: a regex shaped like what you want answers on the first command, a bare word floods on common names, and the wrong flags make a search miss silently.

## Run this

```bash
rg -n -P --no-ignore --hidden -g '!.git' --max-columns 200 --max-columns-preview <scope> '<regex>' .
```

Fill `<scope>` and `<regex>` from the templates below. Example, who calls `my_function` in Python (the calls, not the definition):

```bash
rg -n -P --no-ignore --hidden -g '!.git' --max-columns 200 --max-columns-preview -t py '(?<!def )\bmy_function\s*\(' .
```

Scope: `-t rust`, `-t py`, `-g '*.ts' -g '*.tsx'`, `-g '*.js' -g '*.jsx'`. Where they exist, also add `-g '!node_modules' -g '!target' -g '!dist' -g '!.venv'`.

## Gotchas, most costly first

1. **Always give a path** (`.` is enough). Without one, `rg` reads stdin whenever stdin is a pipe and appears to hang.
2. **Two separate ignore filters.** Gitignored folders and hidden folders are different filters: `--no-ignore` does not unhide dot-folders such as `.github/` or `.env`, and `--hidden` does not stop honoring `.gitignore`. The command above turns both off and excludes `.git`.
3. **An empty or short result is not an answer.** Several search tools return nothing, with no message, for files under a gitignored folder. A class hierarchy, a macro-generated item or a macro argument returns a partial result that looks complete. Confirm with a second route: another regex, or `ast-grep`.
4. **A bare word floods; search for the shape.** A definition, an implementor, a caller and a struct literal each have syntax around the name. Put it in the regex and exclude the other uses with a negative lookbehind.
5. **Nothing caps the output.** `-m` is per file and there is no global limit. When you cannot predict the size, count or list first (`-c`, `-l`), and cap with `| head -N`.
6. **Scope the path** when the tree holds unrelated trees (cloned repositories, vendored code), or their hits drown yours.

## Templates

Text, topics and files:

- Exact text: `-F 'text'`. Only the file names that contain it: `-l`.
- Where a topic lives, when you know no name: `rg -c -i --no-ignore --hidden -g '!.git' 'word1|word2' . | sort -t: -k2 -rn | head -10`, then read the top files.
- A file by name: `rg --files --hidden -g '!.git' -g '**/name'`.

Code shapes (NAME is the symbol; check your language's keywords, these cover Rust, TS/JS and Python):

- Definition, Rust: `\b(fn|struct|enum|trait|type|const|static|mod|union|macro_rules!)\s+NAME\b`
- Definition, TS/JS: `\b(function\*?|class|interface|type|enum|const|let|var)\s+NAME\b`
- Definition, Python: `\b(def|class)\s+NAME\b`
- Implementors, Rust: `\bimpl(<[^>]*>)?\s+([\w:]+::)?TRAIT(<[^>]*>)?\s+for\b` (covers generic and path-qualified forms)
- Subclasses, Python: `\bclass\s+\w+\([^)]*\bBASE\b[^)]*\)`; repeat for each child found, one call per level
- Call sites, not definitions: `(?<!fn )\bNAME\s*\(` (use `def ` for Python, `function ` for TS/JS)
- Struct literals, not definitions or `impl` headers: `(?<!struct )(?<!impl )\bTYPE\s*\{` (expect a few `-> TYPE {` lines)

## ast-grep: code shape a regex cannot say

Calls with a given argument, a construct nested in another, an `impl` for a trait, a rewrite that must touch only real code. It ignores comments and strings. Always trim its output to one line per match, because the default prints every line of every matched node:

```bash
ast-grep run -l rust -p 'impl MyTrait for $T { $$$ }' --no-ignore vcs --json=stream . \
  | jq -r '"\(.file):\(.range.start.line+1): \(.text|split("\n")[0])"'
```

- Always name the language (`-l rust`, `-l python`, `-l ts`, `-l tsx`); `-l ts` skips `.tsx` files.
- Call it `ast-grep`, never `sg`: on Debian and Arch `sg` is shadow-utils' `newgrp`.
- A pattern that does not fit returns an empty or short result, not an error, and it cannot see inside macro arguments (`assert_eq!(..)`, `vec![..]`). Cross-check a short result with the matching `rg` regex and trust the larger one.

## Beyond text search

A language server gives the most exact answer to "which of two same-named functions does this call use": two calls, symbol search then references. Semantic indexers, ranked-snippet tools and other greps are not needed for exact lookups.

## Navigation

- [ast-grep](Ast_Grep/ast_Grep.md): the patterns that silently fail (supertrait bounds, generics, qualified paths, macro arguments) and the second call that recovers a definition or a path-qualified implementor. Read it when an `ast-grep` pattern returns nothing or fewer results than expected.
