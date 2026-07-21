# Improved_Camel_Snake — The Alternative Naming Convention

A readable alternative to standard conventions for code, files, and folders across all languages. Use it when the codebase has no established casing yet and the developer wants to read the code themselves.

## The idea

Standard conventions are flat — every word looks equal weight, the eye has to parse left to right to find what matters. The improved version adds capitalization as semantic signal: **nouns and significant concepts get a capital, verbs and connectives stay lowercase**. Underscores provide the separation.

```
// flat — every word equal weight
compute_series_nav    routing_factory    doc_start.md

// improved — noun stands out
compute_Series_Nav    routing_Factory    doc_Start.md
```

This convention applies consistently across all contexts: code identifiers, file names, and folder names. The only variation is a first-letter rule that differs by context (see below).

---

## Code identifiers (JS/TS, Python)

| Element | Rule | Example |
|---|---|---|
| Variables | noun/object capitalized | `series_All`, `lang_Prefix`, `cur_Idx` |
| Functions | verb lowercase, noun/object capitalized | `compute_Series_Nav()`, `strip_Translation()` |
| Classes | every word capitalized | `Series_Nav_Factory` |
| Constants | every word capitalized | `Max_Retry_Count` |

---

## File names

| Context | First letter | Rule | Example |
|---|---|---|---|
| Code files | follows noun/verb logic | noun-first → uppercase; verb-first → lowercase | `Blog_Linker.js`, `routing_Factory.ts` |
| Non-code files (docs, config, markdown) | always lowercase | first word lowercase, subsequent nouns capitalized | `doc_Start.md`, `writing_Rules.md` |

---

## Folder names

All folders — code or non-code — always start with an uppercase letter. Every word capitalized, underscores between:

```
Browser_Client/    Multi_Lingual/    Blog/    Project_Manag/
```

No exceptions.

---

## Examples by language

### JS / TS

```ts
// variables
const series_All = [];
const lang_Prefix = 'en/';

// functions
function compute_Series_Nav() {}
function strip_Translation(id: string) {}

// files
routing_Factory.ts      // verb-first utility → lowercase start
Blog_Linker.js          // noun/feature → uppercase start
Series_Nav_Factory.js   // noun/feature → uppercase start

// folders
Browser_Client/    Common/    Astro_Frontmatter/
```

### Python

```python
# variables
series_All = []
lang_Prefix = 'en/'

# functions
def compute_Series_Nav(): ...
def strip_Translation(id): ...

# files
routing_Factory.py      # verb-first → lowercase
Blog_Linker.py          # noun → uppercase

# folders
Browser_Client/    Common/
```

### Markdown / docs

```
doc_Start.md          # non-code file → first letter lowercase
writing_Rules.md
linker_File_Structure.md

Project_Manag/        # folder → always uppercase
Architecture/
Brand/
```

---

## Comparison

| Style | Variables | Functions | Files | Readability |
|---|---|---|---|---|
| Standard JS (`camelCase` / `kebab-case`) | `seriesAll` | `computeSeriesNav()` | `routing-factory.ts` | Word boundaries invisible in long names |
| Standard Python (`snake_case`) | `series_all` | `compute_series_nav()` | `routing_factory.py` | Flat — every word equal weight |
| Improved (`improved_Camel_Snake`) | `series_All` | `compute_Series_Nav()` | `routing_Factory.ts` | Noun stands out, word boundaries visible |
