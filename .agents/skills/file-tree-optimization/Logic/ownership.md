# Ownership and responsibility boundaries

Use this guidance to decide where a responsibility belongs and what should stay together. Confirm its purpose, consumers, dependency direction, and lifecycle in the source. A developer working on that subject should be able to predict where its implementation and supporting material live.

Ownership comes before naming or counting. A better home may be elsewhere in the repository; use the closest meaningful owner that covers the actual consumers, while preserving real package and runtime boundaries.

## Keep feature work together

Organize primarily by feature or subject, with one clear entry file and focused supporting files. Reading the entry should explain the feature's work; following its imports should reveal the details. Keep tests beside their implementation. A developer changing one subject should find its fields, schema, validation, feedback, tests, and summaries in predictable places.

Check actual consumers first. A reusable form control with multiple independent users belongs with shared UI, even when this feature happens to use it. Likewise, retain a real server/client subdivision inside a feature when it protects the runtime boundary. Feature-first organization does not forbid technical subdivisions where they carry useful meaning.

The Questions refactor illustrates a complete subject. Education's section, schema, validation, and advisory rules had been spread across editor, domain, and feedback areas. This simplified resulting tree makes the subject discoverable in one place:

```text
Questions/
└── About_You/
    └── Education/
        ├── Feedback/
        │   ├── Rules.ts
        │   └── Rules.test.ts
        ├── Formal/
        │   ├── Completed_Fields.tsx
        │   └── Current_Fields.tsx
        ├── Qualifications/
        │   ├── Field.tsx
        │   ├── Summary.tsx
        │   └── Values.ts
        ├── Schema.ts
        ├── Section.tsx
        └── Validation.ts
```

The meaningful task is changing education behavior. The different file types participate in that task, so their common subject is the useful organizing principle. This is a partial example, not a required schema for every subject; keep genuinely cross-subject primitives with their shared owner.

## Keep sharing within its actual scope

An authentication layout or password input used by several authentication screens can live in `Authentication/Shared/`. Multiple callers alone do not make it an application-wide building block. Choose the closest meaningful owner that covers its actual consumers.

For logic used across the repository, choose an accessible shared home rather than leaving it deep inside one feature. For example, general formatting used by profile, matching, and administration screens can belong under `Shared/Formatting/`. Consumers should call its clear public interface as a black box, without reaching into internal implementation files or copying its behavior. Keep meaningful subdivisions inside that shared home as it grows; accessible placement does not require one giant flat utility folder.

Email illustrates a reusable capability with several specific consumers:

```text
Email/
├── Core/
│   ├── Send.server.ts
│   └── Presentation/
└── Tasks/
    ├── Access_Link/
    ├── Review/
    └── Match/
```

Tasks use Core's sending and rendering. Core stays independent of particular messages. This lets a developer find common sending behavior once and each message's specific work under its task. `Core` here means shared within email; it does not imply a new package or reuse across unrelated applications.

## Keep different lifecycle meanings distinct

```text
Profile/
└── Workspace/
    ├── Live_Draft/
    ├── Submissions/
    └── Review/
        └── Notes/
```

A live draft is editable, a submission is an immutable review snapshot, and review notes are internal to a review. All involve profile data, but their rules differ. Separate homes let a developer recognize those rules before reading implementation details.

Similarly, editable profile notes and internal review notes should not collapse into a generic `Notes/` owner because they share a noun. Inspect meaning, allowed changes, and consumers before combining similarly named concepts.

## Extract a complete job while preserving coordination

The document-upload refactor extracted these responsibilities from larger files:

```text
Upload/
├── Editor.tsx
├── Editor/
│   ├── Errors.ts
│   └── Selection.tsx
├── Store.server.ts
└── Filesystem.server.ts
```

`Editor/Errors.ts` handles upload error classification and wording; `Selection.tsx` owns the file chooser section. `Filesystem.server.ts` owns path, lock, and manifest IO operations. Each extracted file has a complete job that can be named clearly.

The editor retains its coordinating upload state, cancellation/version checks, and object-URL cleanup. The store retains upload lifecycle operations. Keep those relationships clear when extracting collaborators. Splitting consecutive lines or distributing tightly coupled state across arbitrary files makes the workflow harder to follow even if every file meets its size limit.

`Editor.tsx` beside `Editor/` is useful here: the file is the composition entry, and the folder holds its focused supporting work. It earns its place through that responsibility rather than a forwarding layer.

## Align supporting content with its subject owner

The localization refactor separates shared profile labels and questions under `Candidate_Profile/` from wording specific to `Candidate_Portal/` or `Operator_Portal/`. A developer changing an education question can locate the related wording under `Candidate_Profile/Questions/About_You/Education/`.

Use the same subject meanings across implementation, wording, tests, and docs. Keep one authoritative owner for shared material and clear references where trees stay separate. A screen's use of a label does not transfer ownership of that shared label to the screen.

Alignment does not require identical paths or moving all related material into one physical folder. Preserve language authoring, runtime loading, generated-source, and documentation conventions. The relationship should remain predictable within those boundaries.

## A matching word does not override architecture

These names may intentionally remain separate:

```text
Applications/
└── Candidate_Web/
Libraries/
└── Candidate_Contract/
```

The application and reusable library have different package ownership, consumers, and deployment roles. A common `Candidate/` parent that erases those boundaries is not justified by the shared word. A broader workspace reorganization would need evidence and its own scope.

Likewise, root manifests, framework routes, generated outputs, vendored trees, and linked external repositories have placement contracts. Identify those contracts before moving anything; report a genuine conflict with a structural target instead of hiding entries or moving required files just to pass a count.

## Related guidance

- [Grouping and names](grouping.md): consolidate prefixes, review crowded or single-file folders, and remove nesting that adds no meaning.
- [Coding conventions](../../coding/SKILL.md): one responsibility per file, the code-line limit, comments, and established naming styles.
- [Everyday workflow](../SKILL.md): turn source evidence into a scoped proposal, then use Refac and verify the authorized changes.
