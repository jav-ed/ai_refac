import * as fs from "node:fs";
import * as path from "node:path";
import ts from "typescript";

export interface ProjectConfig {
    root: string;
    configPath?: string;
    files: string[];
    options: ts.CompilerOptions;
}

export function readProject(root: string): ProjectConfig {
    const configPath = path.join(root, "tsconfig.json");
    // TypeScript owns config inheritance and include/exclude semantics. Reading
    // configuration creates no Program, language service, or type checker.
    const configured = fs.existsSync(configPath);
    const config = configured
        ? ts.getParsedCommandLineOfConfigFile(configPath, {}, {
            ...ts.sys,
            onUnRecoverableConfigFileDiagnostic(diagnostic) {
                throw new Error(ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"));
            },
        })
        : ts.parseJsonConfigFileContent({ compilerOptions: { allowJs: true, moduleResolution: "bundler", module: "esnext" } }, ts.sys, root);
    if (!config || config.errors.length) {
        throw new Error(`Cannot read TypeScript project ${configPath}: ${config?.errors.map(d => ts.flattenDiagnosticMessageText(d.messageText, "\n")).join("\n")}`);
    }
    if (config.projectReferences?.length) {
        throw new Error("Oxc moves currently require one owning tsconfig without project references; cross-project moves need an explicit project graph.");
    }
    return { root, configPath: configured ? configPath : undefined, files: config.fileNames, options: config.options };
}

export function isSource(file: string): boolean {
    return /\.(?:[cm]?[jt]sx?)$/.test(file);
}
