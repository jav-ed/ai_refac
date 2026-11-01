import * as path from "node:path";

// Without a tsconfig.json TypeScript reads every file under the root, vendored trees and build
// output included, and the engine then parses all of them. git knows which files the project
// ignores: its tracked files plus the untracked ones it does not ignore (a file created a minute
// ago counts). Returns undefined when `root` is not inside a git work tree, because then nothing
// says what is ignored. Any other git failure is an error: guessing would scan the wrong files.
export function unignoredFiles(root: string): Set<string> | undefined {
    let result: ReturnType<typeof Bun.spawnSync>;
    try {
        result = Bun.spawnSync(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], {
            cwd: root,
            // The "not a git repository" check below reads git's English message.
            env: { ...process.env, LC_ALL: "C" },
            stdout: "pipe",
            stderr: "pipe",
        });
    } catch (error) {
        throw new Error(`Cannot list the files of ${root}: ${root}/tsconfig.json is missing, so refac asks git which files the project ignores, and git did not start (${error}). Install git or add a tsconfig.json to the project root.`);
    }
    if (result.exitCode !== 0) {
        const message = result.stderr.toString().trim();
        if (/not a git repository/.test(message)) return undefined;
        throw new Error(`Cannot list the files of ${root}: ${root}/tsconfig.json is missing, so refac asks git which files the project ignores, and \`git ls-files\` failed with exit code ${result.exitCode}: ${message}`);
    }
    return new Set(result.stdout.toString().split("\0").filter(Boolean).map(file => path.resolve(root, file)));
}
