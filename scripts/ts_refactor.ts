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

async function getProject(projectRoot: string) {
    const tsConfigPath = path.join(projectRoot, "tsconfig.json");
    let project: Project;

    if (fs.existsSync(tsConfigPath)) {
        project = new Project({
            tsConfigFilePath: tsConfigPath,
        });
    } else {
        project = new Project({
            compilerOptions: {
                allowJs: true,
            }
        });
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

    const project = await getProject(projectRoot);

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

    await project.save();
    console.log("Move successful");
}

async function performBatchMove(fileMap: any[], projectRoot: string) {
    const project = await getProject(projectRoot);
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

    await project.save();
    console.log(`Batch move completed. Moved ${successCount} items.`);
}

main().catch(e => {
    console.error(e);
    process.exit(1);
});
