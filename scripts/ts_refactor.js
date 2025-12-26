const { Project } = require("ts-morph");
const path = require("path");
const fs = require("fs");

async function main() {
    const args = process.argv.slice(2);
    if (args.length < 1) {
        console.error("Usage: node ts_refactor.js <command> [args]");
        process.exit(1);
    }
    const command = args[0];
    const projectRoot = args[args.length - 1].startsWith("/") ? args[args.length - 1] : process.cwd();

    if (command === "move") {
        if (args.length < 3) {
            console.error("Usage: node ts_refactor.js move <source> <target> [project_root]");
            process.exit(1);
        }
        const sourcePath = args[1];
        const targetPath = args[2];
        await performMove(sourcePath, targetPath, projectRoot);
    } else if (command === "batch") {
        if (args.length < 2) {
            console.error("Usage: node ts_refactor.js batch <json_payload> [project_root]");
            process.exit(1);
        }
        const payload = args[1];
        let fileMap;
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

async function getProject(projectRoot) {
    const tsConfigPath = path.join(projectRoot, "tsconfig.json");
    let project;

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

async function performMove(sourcePath, targetPath, projectRoot) {
    const absSource = path.resolve(projectRoot, sourcePath);
    const absTarget = path.resolve(projectRoot, targetPath);

    console.log(`Moving ${absSource} to ${absTarget}`);

    if (!fs.existsSync(absSource)) {
        console.error(`Source file not found: ${absSource}`);
        process.exit(1);
    }

    const project = await getProject(projectRoot);
    const sourceFile = project.getSourceFile(absSource);

    if (!sourceFile) {
        console.error(`Source file not found in project context: ${absSource}`);
        process.exit(1);
    }

    await sourceFile.move(absTarget);
    await project.save();
    console.log("Move successful");
}

async function performBatchMove(fileMap, projectRoot) {
    // fileMap is Array of {source: string, target: string} or objects?
    // Let's assume Array of [source, target] tuples or similar structure from Rust.
    // The previous design sends Vec<(String, String)>, which serializes to [[s,t], [s,t]].

    const project = await getProject(projectRoot);
    let successCount = 0;

    for (const item of fileMap) {
        // Handle both object {source, target} and tuple [source, target]
        let sourcePath, targetPath;
        if (Array.isArray(item)) {
            sourcePath = item[0];
            targetPath = item[1];
        } else {
            sourcePath = item.source;
            targetPath = item.target;
        }

        const absSource = path.resolve(projectRoot, sourcePath);
        const absTarget = path.resolve(projectRoot, targetPath);

        const sourceFile = project.getSourceFile(absSource);
        if (!sourceFile) {
            console.warn(`Skipping missing source in batch: ${absSource}`);
            continue;
        }

        try {
            sourceFile.move(absTarget);
            successCount++;
        } catch (e) {
            console.error(`Failed to move ${absSource} -> ${absTarget}: ${e.message}`);
        }
    }

    await project.save();
    console.log(`Batch move completed. Moved ${successCount} files.`);
}

main().catch(e => {
    console.error(e);
    process.exit(1);
});
