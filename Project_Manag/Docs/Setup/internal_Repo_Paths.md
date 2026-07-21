# Internal Repository Paths

The CLI and its public documentation are independent first-party repositories kept as siblings.

## Local Layout

```text
01_Refac/
  README.md
  Refac_Cli_Code/
  Refac_Docs/
```

- This repository owns CLI implementation, tests, engineering docs, and the `refac-cli` skill.
- The sibling `../Refac_Docs/` repository owns the public Fumadocs website.

When verified CLI behavior changes, update the corresponding public documentation in `Refac_Docs`. Do not place either repository under the ignored `Repos/` folder; that folder is for upstream reference checkouts.
