1. It can't be stressed enough that file-folder structure must be clear and intuitive. This means a lot. To start, let's include other terms you might know better - file-folder structure could be understood as tree view, directory tree, or high-level project architecture. The different terminology inshallah helps to see the bigger picture.
2. the tree-view should make it obvious-what's the entry point / file, which files are the main files, which are helping files. the selection of file and folder name helps for that. In summary: the information flow (input/output, dependencies) should become obvious.

Some Helpers:

1. Before you can think about coding, or even the file-folder structure, have deep understanding of the goal. If the goal is clear, understand the current code basis. For this initial task you need to think hard
2. Code base and goal are understood, now, you can suggest some tree-views. For that, some clean code helpers should be known

Clean Code helper:

1. Obvious Code: Make inputs and outputs crystal clear. Avoid unnecessary abstractions - keep methods, classes, and functions straightforward
2. No backward compatibility (if error occurs, it should break the code) - single code with clear input/output structure
3. Modular Organization: For each feature, create a main file (descriptive filename with suffix main) alongside a supporting folder. Inside this folder, put all the helper files that the main file uses. For complex tasks that can be separated in clear subfeatures, this pattern can be applied for deeper lvl organization. This makes the entry point and dependencies immediately clear. It makes it easy to read, understand and expand the code.

How to know when to create a new main?

1. if you know the goal, you can assess which features are required
2. can these features be broken into sub features?
3. how much code these feature need. if a single file would need more than 300 lines of code, it for sure needs to be broken into either main + folder, break into multiple helping files and move into already existing (subfolders), or even create new helping (sub)folders.
4. One essential task is to decide when to create a new main file with its helping folders, and when to create new helping folders for an already existing main file. 5. The main file makes the entry point obvious, the accompanying (sub)folder are grouped by purpose.

Being honest about work
One the most important thing you really need to understand and accept: you and your working environment loves productivty and effiency. For that the key is honesty - honesty in work ethics and communication

1. If you dont undestand - say it, let it be known. this helps throughout each single process
2. if you have questions - ask,
3. if you couldn't complete/solve or had any other troubles - be honest and say it. It inshallah will be rewarding. We are able to find bugs/issues earlier - can adapt plans as needed, can get additional professionals in to help solve issue, other developers, friends and your manager will thank you,....

Here again, thinking is required. As the task sometimes cannot be solved in a one-shot approach:

1. think of multiple tree-views
2. assess them all
3. find the favorite
