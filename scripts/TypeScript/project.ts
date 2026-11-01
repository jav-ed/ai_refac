import * as fs from "node:fs";
import * as path from "node:path";
import ts from "typescript";
import { unignoredFiles } from "./unignored";

export interface ProjectConfig {
    root: string;
    configPath?: string;
    files: string[];
    options: ts.CompilerOptions;
}

// Component and style files that name other files. TypeScript lists them in a project only
// when told, the way an editor tells it about .vue files; their code is read by
// Embedded/, never by the compiler.
const COMPONENT_EXTENSIONS: ts.FileExtensionInfo[] = [
    { extension: ".astro", isMixedContent: true, scriptKind: ts.ScriptKind.Deferred },
    { extension: ".mdx", isMixedContent: true, scriptKind: ts.ScriptKind.Deferred },
    // A stylesheet names files with url() and @import.
    { extension: ".css", isMixedContent: false, scriptKind: ts.ScriptKind.Deferred },
    // A page names files in its frontmatter.
    { extension: ".md", isMixedContent: true, scriptKind: ts.ScriptKind.Deferred },
];

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
        }, undefined, undefined, COMPONENT_EXTENSIONS)
        : ts.parseJsonConfigFileContent({ compilerOptions: { allowJs: true, moduleResolution: "bundler", module: "esnext" } }, ts.sys, root, undefined, undefined, undefined, COMPONENT_EXTENSIONS);
    if (!config || config.errors.length) {
        throw new Error(`Cannot read TypeScript project ${configPath}: ${config?.errors.map(d => ts.flattenDiagnosticMessageText(d.messageText, "\n")).join("\n")}`);
    }
    if (config.projectReferences?.length) {
        throw new Error("Oxc moves currently require one owning tsconfig without project references; cross-project moves need an explicit project graph.");
    }
    // A tsconfig.json says itself which files belong to the project. Without one, TypeScript's
    // default is every file under the root, so the files the project ignores are dropped.
    let files = config.fileNames;
    if (!configured) {
        const unignored = unignoredFiles(root);
        if (unignored) files = files.filter(file => unignored.has(path.resolve(file)));
    }
    return { root, configPath: configured ? configPath : undefined, files, options: config.options };
}

export function isSource(file: string): boolean {
    return /\.(?:[cm]?[jt]sx?|astro|mdx?|css)$/.test(file);
}
