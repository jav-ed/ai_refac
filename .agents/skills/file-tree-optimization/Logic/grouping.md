# Purposeful grouping and names

Use these rules when repeated names, crowded folders, or empty layers make related work hard to find. Inspect entry files, callers, and documentation before regrouping: similar names suggest possibilities, while the code establishes the relationship.

Every parent should explain what its children share, and each child name should explain its local purpose. Each path segment must add useful context. These examples illustrate decisions, not automatic rewriting patterns; leave a clear tree unchanged when no supported improvement is evident.

## Repeated prefixes can identify a missing parent

Before, three siblings implement parts of one candidate domain:

```text
Features/
├── Candidate_Access_Link/
├── Candidate_Consent/
└── Candidate_Profile/
```

After confirming that shared domain in the source:

```text
Features/
└── Candidate/
    ├── Access_Link/
    ├── Consent/
    └── Profile/
```

The prefix describes the common domain. The suffixes describe the children's distinct responsibilities. Moving the prefix into a parent supplies the context once, reduces the number of competing entries under `Features`, and makes a new candidate feature's home easier to predict.

| Old path | New path |
| --- | --- |
| `Features/Candidate_Access_Link/` | `Features/Candidate/Access_Link/` |
| `Features/Candidate_Consent/` | `Features/Candidate/Consent/` |
| `Features/Candidate_Profile/` | `Features/Candidate/Profile/` |

The number three is not a threshold. Two related folders can justify a parent; five similarly named folders can still belong apart. If `Candidate/` already exists, inspect its ownership and children before reusing it. Check every destination for collisions. Do not silently merge two different `Profile` responsibilities.

Ownership can override the prefix. In the application's matching refactor, `Candidate_Participation/` became `Matching/Participation/`: its eligibility and participation history belong to matching. Inspect what the code does and who uses it before deciding that every Candidate-prefixed folder belongs under `Candidate/`.

## Remove context that the path already supplies

Before:

```text
Candidate/
└── Profile/
    ├── Candidate_Profile_Editor.tsx
    ├── Candidate_Profile_Validation.ts
    └── Candidate_Profile_Validation.test.ts
```

After:

```text
Candidate/
└── Profile/
    ├── Editor.tsx
    ├── Validation.ts
    └── Validation.test.ts
```

The parent path already says which profile is involved. Local names can focus on the file's role. Keep the implementation and its test recognizable as a pair, and follow the repository's casing convention.

Remove semantic repetition, not every matching substring. `Access/Access_Level.ts` may express a meaningful concept whose shortened name `Level.ts` is less clear. Preserve required filenames and runtime suffixes such as `.server.ts`; putting a file inside `Server/` does not replace a framework's filename contract. Exported identifiers may still need domain names for clarity at their call sites.

## Split a crowded folder by responsibility

More than nine direct files is a strong signal to inspect the folder for meaningful subdivisions or misplaced responsibilities. Before, ten direct files mix the editor, contact form, and preferences form:

```text
Profile/
├── Editor.tsx
├── Contact_Email.tsx
├── Contact_Phone.tsx
├── Contact_Validation.ts
├── Contact_Validation.test.ts
├── Preferences_Fields.tsx
├── Preferences_Choices.ts
├── Preferences_Validation.ts
├── Preferences_Validation.test.ts
└── Preferences_Summary.tsx
```

If the implementation confirms two cohesive form responsibilities:

```text
Profile/
├── Contact/
│   ├── Email.tsx
│   ├── Phone.tsx
│   ├── Validation.ts
│   └── Validation.test.ts
├── Preferences/
│   ├── Fields.tsx
│   ├── Choices.ts
│   ├── Validation.ts
│   ├── Validation.test.ts
│   └── Summary.tsx
└── Editor.tsx
```

The parent has one direct file plus two subfolders; the children have four and five files. Each group has a purpose, tests remain beside their implementation, and the editor remains the clear entry point. Subdirectories do not count as files, but a directory with many unrelated subfolders still deserves an ownership review. A single entry file beside meaningful supporting subfolders is different from an otherwise empty folder around one file.

`Group_1/` and `Group_2/` would reduce the count without improving understanding. Moving all tests into a distant `Tests/` folder merely to lower the count would also miss the point. Do not merge unrelated files or create a folder for every small helper to satisfy a numeric target.

The correct move can leave this area entirely. If inspection finds reusable number formatting inside a crowded `Profile/` folder, and independent features use the same logic, move that responsibility to an appropriate shared formatting owner elsewhere in the repository. Do not force it into `Profile/Helpers/` because that is near its current location. Current placement is evidence to examine, not a boundary that the new organization must preserve.

## Remove empty layers, preserve meaningful boundaries

`Profile/UI/Components/Editor.tsx` can become `Profile/Editor.tsx` if `UI/Components` supplies no ownership, runtime, or navigation distinction and the parent remains clear. Do not flatten it when `UI` deliberately separates browser code from server implementation or when the subtree has meaningful responsibilities of its own.

Do not create a layer or folder solely to hold one trivial forwarding file.

A single-file folder is a strong clue to review its purpose. A meaningful feature boundary or planned related additions can justify keeping it. Give extra attention to several such folders together: they may reveal one useful group that the current names obscure.

Before, three folders each wrap one formatting file:

```text
Formatting/
├── Currency/
│   └── Format.ts
├── Date/
│   └── Format.ts
└── Number/
    └── Format.ts
```

If those wrappers add no useful boundary and no related additions are planned, rename and regroup:

```text
Formatting/
├── Currency.ts
├── Date.ts
└── Number.ts
```

The shared purpose is visible once, and each file tells the developer which formatting behavior it owns. Related files can also come from different parts of the repository; inspect their responsibilities before selecting a new common home. Do not combine unrelated single-file folders just because their counts match.

## Related guidance

- [Ownership and boundaries](ownership.md): choose the right home, keep complete subjects together, and separate shared core or different lifecycles before regrouping.
- [Exact structural counts](../Usage/counts.md): inspect each folder's direct files and subfolders with explicit hidden and ignored-path scope.
- [Coding conventions](../../coding/SKILL.md): established naming styles, the code-line limit, and comment preservation.
- [Everyday workflow](../SKILL.md): inspect, present a before/after proposal, execute authorized moves, and verify the result.
