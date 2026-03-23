---
name: coding
description: coding conventions
---

# Coding Standards

## Instructions

1. Commenting is very important. During coding, comments help to understand multiple things: what are we doing, and why are we doing it like that (mindset, design decisions). Adding comments allows us to get a deeper understanding of the task, potentially also helping us spot shortcomings while coding. Additionally, it is a must so other devs can understand what and how we did it.
   - Special demand on how we add comments:

     ```
     // comment here
     first_Code

     // second comment again
     second_Code
     ```

   - The comment is directly followed by the code line it addresses. The blank line between blocks gives a clear separation of which comment belongs to which code.
2. We have a very efficient way of writing code, following a special camel_Case_With_Underline style for everything. The only expection is that we use Big_Letter_Camel_Case_With_Underline for folders. For more see  .agent/skills/naming-conventions/SKILL.md
3. Obvious Code: Make inputs and outputs crystal clear. Avoid unnecessary abstractions - keep methods, classes, and functions straightforward
4. It can't be stressed enough that folder/file structure should be clear, intuitive.
5. Not more than 300 lines per file (you cannot remove comments to achieve this goal)

### Special helper for js

1. if we are on an astro project, you should have access to the dev-tools mcp. you can check localhost 4321 to get screenshots of what the Frontend
