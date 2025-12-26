const { Project } = require("ts-morph");
const path = require("path");
const fs = require("fs");

async function main() {
    const args = process.argv.slice(2);
    if (args.length < 2) {
        console.error("Usage: node ts_refactor.js <source> <target> [project_root]");
        process.exit(1);
    }

    const sourcePath = args[0];
    const targetPath = args[1];
    const projectRoot = args[2] || process.cwd();

    // ensure paths are absolute
    const absSource = path.resolve(projectRoot, sourcePath);
    const absTarget = path.resolve(projectRoot, targetPath);

    console.log(`Moving ${absSource} to ${absTarget}`);

    if (!fs.existsSync(absSource)) {
         console.error(`Source file not found: ${absSource}`);
         process.exit(1);
    }

    // Initialize ts-morph project
    // We try to find tsconfig, otherwise default
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

    const sourceFile = project.getSourceFile(absSource);
    if (!sourceFile) {
        // If not in project, try adding it
        console.error(`Source file not found in project context: ${absSource}`);
        // Attempt to add?
        process.exit(1);
    }

    // Perform move
    // ts-morph handles imports automatically
    await sourceFile.move(absTarget);
    
    // Save changes
    await project.save();
    
    console.log("Move successful");
}

main().catch(e => {
    console.error(e);
    process.exit(1);
});
