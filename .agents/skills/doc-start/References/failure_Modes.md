# Failure modes

Ways the navigation-first pattern gets misapplied to repo docs.

## Pure-index linker

A linker file that opens straight into a list of links, with no summary of the area. The reader is forced to open every link to know what is relevant. Every linker (and `doc_Start.md`) opens with a short summary first, so the reader can skip the whole area when it does not apply.

## Leaf enumeration in a parent

When a sub-folder has its own linker, the parent should point to that sub-linker, not list every leaf inside it. Enumerating leaves in the parent rots fast (every new leaf needs an update in two places) and pushes the routing decision back to the parent instead of the area that owns it. The parent points to the door, the sub-linker handles the room.

## Dumping into doc_Start.md

`doc_Start.md` is not a sitemap of every doc in the repo. Including everything makes the entry page longer to scan and gives no signal about what to read first. Include only what the agent needs to decide where to go next; route everything else through linkers.

## Vague link descriptions

"See the architecture doc" is not a description, it is an invitation to open the file. The reader needs to know what kinds of tasks or questions belong behind the link before clicking. Bad labels make a well-organized doc tree worse than a flat one, because the cost of routing rises without the benefit of skipping irrelevant material.

## Imposed reading order

"Start here first" or "read these in order" takes agency away from the reader. The agent already knows what task it is on and does not need to be told the order. Only impose a reading order when the user explicitly asks for one.

## Stale linkers

Linkers rot when docs are added, renamed, or moved without updating the linker. Before trusting or editing an existing linker, glob the folder it lives in, verify each entry still points to a real file, and verify each sibling doc is listed. Spot-check that descriptions still match what the linked file actually contains.

## Duplicate content

The same explanation copied into two files. Each copy then has to be maintained, and copies drift. One file owns a topic; everywhere else, link to it.
