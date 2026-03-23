---
name: naming-conventions
description: Enforce camel_Case_With_Underline naming conventions for coding, that is, creating file names, variables, constants, class names, folder names and you name it
---

# Naming Conventions & Coding Standards

## Instructions
We use a very perfomant and not generally known naming convention. For nearly everything (file names, variables, constants, class names and you name it) we use camel_Case_With_Underline. Only and only for folder we start with a big letter, like This_Is_A_Folder_Name ( which is also very close to the camel_Case_With_Underline convention).
While this is close to snake_case, camel_Case_With_Underline is a bit different.


### Special Cases
1. for booleans, we start the variable with the letter b, like in b_This_Is_A_Boolean (only for booleans, not for strings, dicts, etc.)

**Summary**:
- Files/variables/functions/classes/constants: `small_Starting_But_Then_Big`
- Folders (THE ONLY EXCEPTION): `Big_Starting_And_Remaining_Big`
- Entry files: `small_Starting_But_Then_Big_Entr`
- Max 300 lines per file
- No fallbacks - hard breaks only. If things crash, we need to experince those crashes.

**The One Rule**: Everything starts lowercase EXCEPT folders which start uppercase.
