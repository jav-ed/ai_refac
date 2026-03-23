import { Project, SourceFile, Directory } from "ts-morph";
import * as path from "path";
import * as fs from "fs";

// Bun uses ES modules by default

async function main() {
    const args = process.argv.slice(2);
    if (args.length < 1) {
        console.error("Usage: bun ts_refactor.ts <command> [args]");
        process.exit(1);
    }
    const command = args[0];
    const lastArg = args[args.length - 1];
    const projectRoot = (lastArg && lastArg.startsWith("/")) ? lastArg : process.cwd();

    if (command === "move") {
        if (args.length < 3) {
            console.error("Usage: bun ts_refactor.ts move <source> <target> [project_root]");
            process.exit(1);
        }
        const sourcePath = args[1];
        const targetPath = args[2];
        await performMove(sourcePath, targetPath, projectRoot);
    } else if (command === "batch") {
        if (args.length < 2) {
            console.error("Usage: bun ts_refactor.ts batch <json_payload> [project_root]");
            process.exit(1);
        }
        const payload = args[1];
        let fileMap: any;
        try {
            fileMap = JSON.parse(payload);
        } catch (e) {
            console.error("Invalid JSON payload for batch operation");
            process.exit(1);
        }

        await performBatchMove(fileMap, projectRoot);
    } else {
        console.error("Unknown command:", command);
        console.log("Supported commands: move, batch");
        process.exit(1);
    }
}

const LARGE_PROJECT_FILE_THRESHOLD = 500;

function isDirectory(p: string): boolean {
    try { return fs.statSync(p).isDirectory(); } catch { return false; }
}

async function getProject(projectRoot: string, filesToMove?: string[]) {
    const tsConfigPath = path.join(projectRoot, "tsconfig.json");

    // Directory moves always need the full project so ts-morph can find and update
    // external importers. Skip the large-project fast path in that case.
    const hasDirMove = (filesToMove ?? []).some(isDirectory);

    if (fs.existsSync(tsConfigPath)) {
        if (hasDirMove) {
            process.stderr.write(`[refac] Initializing full TypeScript project for directory move (this may take a moment for large projects)...\n`);
            return new Project({ tsConfigFilePath: tsConfigPath });
        }

        // File moves: count first — loading a 10k+ file project via tsconfig causes
        // multi-minute freezes with no output.
        // Uses Bun's built-in Glob — no npm dependency required.
        const SKIP_DIRS = new Set(["node_modules", "dist", "build", ".next", ".git"]);
        const bunGlob = new Bun.Glob("**/*.{ts,tsx,js,jsx}");
        const allFiles: string[] = [];
        for await (const file of bunGlob.scan({ cwd: projectRoot, absolute: true })) {
            const parts = file.split(path.sep);
            if (!parts.some(p => SKIP_DIRS.has(p))) {
                allFiles.push(file);
            }
        }

        if (allFiles.length > LARGE_PROJECT_FILE_THRESHOLD) {
            process.stderr.write(
                `[refac] Large project detected (${allFiles.length} files). Loading only moved files for performance. Cross-project reference updates will be skipped.\n`
            );
            const project = new Project({ compilerOptions: { allowJs: true, skipLibCheck: true } });
            for (const f of filesToMove ?? []) {
                if (fs.existsSync(f) && !isDirectory(f)) {
                    project.addSourceFileAtPath(f);
                }
            }
            return project;
        }

        process.stderr.write(`[refac] Initializing TypeScript project (${allFiles.length} files)...\n`);
        return new Project({ tsConfigFilePath: tsConfigPath });
    }

    // No tsconfig — glob everything
    const project = new Project({ compilerOptions: { allowJs: true } });
    if (hasDirMove) {
        // For directory moves without tsconfig, load all TS files so references can be found
        project.addSourceFilesAtPaths(path.join(projectRoot, "**/*{.ts,.tsx,.js,.jsx}"));
    } else {
        project.addSourceFilesAtPaths(path.join(projectRoot, "**/*{.ts,.tsx,.js,.jsx}"));
    }
    return project;
}

async function performMove(sourcePath: string, targetPath: string, projectRoot: string) {
    const absSource = path.resolve(projectRoot, sourcePath);
    const absTarget = path.resolve(projectRoot, targetPath);

    console.log(`Moving ${absSource} to ${absTarget}`);

    if (!fs.existsSync(absSource)) {
        console.error(`Source file not found: ${absSource}`);
        process.exit(1);
    }

    const targetDir = path.dirname(absTarget);
    if (!fs.existsSync(targetDir)) {
        fs.mkdirSync(targetDir, { recursive: true });
    }

    const project = await getProject(projectRoot, [absSource]);

    // Check if directory
    let isDir = false;
    try {
        isDir = fs.statSync(absSource).isDirectory();
    } catch { }

    if (isDir) {
        const directory = project.getDirectory(absSource);
        if (directory) {
            directory.move(absTarget);
        } else {
            // Try add
            project.addDirectoryAtPath(absSource).move(absTarget);
        }
    } else {
        const sourceFile = project.getSourceFile(absSource);
        if (!sourceFile) {
            console.error(`Source file not found in project context: ${absSource}`);
            process.exit(1);
        }
        sourceFile.move(absTarget);
    }

    process.stderr.write("[refac] Saving changes...\n");
    await project.save();
    console.log("Move successful");
}

async function performBatchMove(fileMap: any[], projectRoot: string) {
    const sourcePaths: string[] = fileMap.map((item: any) => {
        const raw = Array.isArray(item) ? item[0] : item.source;
        return path.resolve(projectRoot, raw);
    });
    const project = await getProject(projectRoot, sourcePaths);
    let successCount = 0;

    for (const item of fileMap) {
        let sourcePath: string, targetPath: string;
        // Tuple [s, t] or Object {source, target}
        if (Array.isArray(item)) {
            sourcePath = item[0];
            targetPath = item[1];
        } else {
            sourcePath = item.source;
            targetPath = item.target;
        }

        const absSource = path.resolve(projectRoot, sourcePath);
        const absTarget = path.resolve(projectRoot, targetPath);

        let isDir = false;
        try {
            const stats = fs.statSync(absSource);
            isDir = stats.isDirectory();
        } catch (e) {
            console.warn(`Source path not found on disk: ${absSource}`);
            continue;
        }

        const targetDir = path.dirname(absTarget);
        if (!fs.existsSync(targetDir)) {
            fs.mkdirSync(targetDir, { recursive: true });
        }

        try {
            if (isDir) {
                // Directory Move
                const directory = project.getDirectory(absSource);
                if (directory) {
                    directory.move(absTarget);
                    successCount++;
                } else {
                    try {
                        const addedDir = project.addDirectoryAtPath(absSource);
                        addedDir.move(absTarget);
                        successCount++;
                    } catch (addErr: any) {
                        console.error(`Failed to add/move directory ${absSource}: ${addErr.message}`);
                    }
                }
            } else {
                // File Move
                const sourceFile = project.getSourceFile(absSource);
                if (!sourceFile) {
                    if (fs.existsSync(absSource)) {
                        try {
                            project.addSourceFileAtPath(absSource).move(absTarget);
                            successCount++;
                        } catch (addErr: any) {
                            console.warn(`Failed to add/move file ${absSource}: ${addErr.message}`);
                        }
                    } else {
                        console.warn(`Skipping missing source file: ${absSource}`);
                    }
                    continue;
                }
                sourceFile.move(absTarget);
                successCount++;
            }
        } catch (e: any) {
            console.error(`Failed to move ${absSource} -> ${absTarget}: ${e.message}`);
        }
    }

    process.stderr.write("[refac] Saving changes...\n");
    await project.save();
    console.log(`Batch move completed. Moved ${successCount} items.`);
}

main().catch(e => {
    console.error(e);
    process.exit(1);
});
