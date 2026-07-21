# Labels and descriptions

Skill files have two places where text next to something is the routing signal: the frontmatter `description`, and inline link descriptions. In both cases, the description is more important than the thing it describes. Vague descriptions cause the agent to load the wrong skill or fail to skip an irrelevant one, and either way its context fills with content that does not help the current task.

## Frontmatter description

The only thing the agent sees when deciding whether to load the skill. It is surfaced alongside every other installed skill in the system prompt, and the agent picks based on these descriptions alone.

### Format

- Max 1024 characters.
- Third person.
- First sentence: what the skill does.
- Second sentence: `Use when [specific triggers: keywords, contexts, file types, user phrases].`

### Triggers

The "Use when" clause is the routing signal. Be specific. List concrete keywords, file extensions, user phrases, or task types the agent should recognize.

Good triggers:

- "Use when working with PDF files or when user mentions PDFs, forms, or document extraction."
- "Use when the user wants to create, write, or build a new skill."

Bad triggers:

- "Use when you need to handle documents." (vague, overlaps with every other doc skill)
- "Use as needed." (no signal)

### Examples

Good:

```
Extract text and tables from PDF files, fill forms, merge documents. Use when working with PDF files or when user mentions PDFs, forms, or document extraction.
```

Bad:

```
Helps with documents.
```

The bad example gives the agent no way to distinguish this from any other document-related skill.

### Common mistakes

- Selling the skill ("powerful tool for..."): the agent does not need marketing.
- Listing internal mechanics: the description is for routing, not architecture.
- Burying the trigger inside prose: keep "Use when" as a discrete clause.
- Promising more than the skill delivers: misrouting wastes a load.
- Overlapping triggers with other installed skills: if two skills both say "Use when working with files," neither will route reliably.

## Inline link descriptions

Inside SKILL.md and reference files, every link is a routing decision. The text next to the link is what the agent uses to decide whether to follow it.

### Format

```
- [Short label](path.md): description rich enough for the agent to decide without clicking
```

- Use a colon between the link and its description.
- Never use an em-dash anywhere in nav lists.
- The label is short, the description is as long as the topic needs.

### What the description must do

It must let the agent answer "is this relevant to my current task?" without opening the file. So it must say what kinds of tasks, questions, or contexts belong behind the link.

Bad:

```
- [advanced.md](advanced.md): more details
```

Better:

```
- [Concurrency edge cases](advanced.md): rules for parallel reads, lock contention, and the retry behavior used inside the queue worker
```

### Length

One line when one line covers it. Five lines when the topic genuinely needs five. Clarity for navigation, not brevity. If the topic has multiple distinct angles, list them rather than collapsing to a vague summary.

### Common failure modes

- Vague descriptions ("more info", "see file") force the agent to open the file to decide. The whole point of the label is to avoid that.
- Bare filenames as labels ("podman.md") give no signal.
- Marketing labels ("the best way to do X") give no signal either.
- Descriptions that paraphrase the H1 of the target file are wasted: write what is in the file, not what it is called.
- Sub-area linkers should be linked as linkers, not have their leaves enumerated in the parent. The parent points to the door, the sub-linker handles the room.
