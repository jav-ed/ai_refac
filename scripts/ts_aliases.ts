import { Node, Project, StringLiteral, ts } from "ts-morph";
import * as path from "path";

export interface AliasRewrite {
    oldPrefix: string;
    newPrefix: string;
}

interface MovePath {
    source: string;
    target: string;
}

function normalizeModulePath(value: string): string {
    return value.split(path.sep).join("/").replace(/^\.\//, "");
}

function withoutSourceExtension(value: string): string {
    return value.replace(/\.(d\.)?(ts|tsx|js|jsx|mts|cts|mjs|cjs)$/, "");
}

function wildcardCapture(pattern: string, value: string): string | undefined {
    const wildcard = pattern.indexOf("*");
    if (wildcard === -1) return pattern === value ? "" : undefined;
    if (pattern.indexOf("*", wildcard + 1) !== -1) return undefined;

    const prefix = pattern.slice(0, wildcard);
    const suffix = pattern.slice(wildcard + 1);
    if (!value.startsWith(prefix) || !value.endsWith(suffix)) return undefined;
    return value.slice(prefix.length, value.length - suffix.length);
}

function applyCapture(pattern: string, capture: string): string | undefined {
    const wildcard = pattern.indexOf("*");
    if (wildcard === -1) return capture.length === 0 ? pattern : undefined;
    if (pattern.indexOf("*", wildcard + 1) !== -1) return undefined;
    return `${pattern.slice(0, wildcard)}${capture}${pattern.slice(wildcard + 1)}`;
}

function captureForTarget(pattern: string, value: string): string | undefined {
    return wildcardCapture(pattern, value) ?? wildcardCapture(pattern, withoutSourceExtension(value));
}

export function createAliasRewrites(
    project: Project,
    projectRoot: string,
    moves: MovePath[],
): AliasRewrite[] {
    const compilerOptions = project.getCompilerOptions();
    const pathAliases = compilerOptions.paths;
    if (!pathAliases) return [];

    const baseDirectory = compilerOptions.baseUrl
        ? path.resolve(compilerOptions.baseUrl)
        : projectRoot;
    const rewrites = new Map<string, AliasRewrite>();

    for (const [aliasPattern, targetPatterns] of Object.entries(pathAliases)) {
        for (const targetPatternValue of targetPatterns) {
            const targetPattern = normalizeModulePath(targetPatternValue);
            for (const move of moves) {
                const source = normalizeModulePath(path.relative(baseDirectory, move.source));
                const target = normalizeModulePath(path.relative(baseDirectory, move.target));
                const sourceCapture = captureForTarget(targetPattern, source);
                const targetCapture = captureForTarget(targetPattern, target);
                if (sourceCapture === undefined || targetCapture === undefined) continue;

                const oldPrefix = applyCapture(aliasPattern, sourceCapture);
                const newPrefix = applyCapture(aliasPattern, targetCapture);
                if (!oldPrefix || !newPrefix || oldPrefix === newPrefix) continue;
                rewrites.set(`${oldPrefix}\0${newPrefix}`, { oldPrefix, newPrefix });
            }
        }
    }

    return [...rewrites.values()].sort((a, b) => b.oldPrefix.length - a.oldPrefix.length);
}

function matchesPrefix(specifier: string, prefix: string): boolean {
    return specifier === prefix || specifier.startsWith(`${prefix}/`);
}

function isModuleSpecifier(node: StringLiteral): boolean {
    const literal = node.compilerNode;
    const parent = literal.parent;
    if (ts.isImportDeclaration(parent) || ts.isExportDeclaration(parent)) {
        return parent.moduleSpecifier === literal;
    }
    if (ts.isExternalModuleReference(parent)) return parent.expression === literal;
    if (ts.isCallExpression(parent) && parent.arguments[0] === literal) {
        return parent.expression.kind === ts.SyntaxKind.ImportKeyword
            || (ts.isIdentifier(parent.expression) && parent.expression.text === "require");
    }
    return ts.isLiteralTypeNode(parent) && ts.isImportTypeNode(parent.parent);
}

export function rewriteAliasModuleSpecifiers(project: Project, rewrites: AliasRewrite[]): void {
    if (rewrites.length === 0) return;

    for (const sourceFile of project.getSourceFiles()) {
        // Collect before editing so replacing literals cannot invalidate traversal state.
        const specifiers = sourceFile
            .getDescendantsOfKind(ts.SyntaxKind.StringLiteral)
            .filter(isModuleSpecifier);

        for (const specifier of specifiers) {
            const current = specifier.getLiteralValue();
            const rewrite = rewrites.find(candidate => matchesPrefix(current, candidate.oldPrefix));
            if (!rewrite) continue;
            specifier.setLiteralValue(`${rewrite.newPrefix}${current.slice(rewrite.oldPrefix.length)}`);
        }
    }
}

export function assertNoStaleAliasSpecifiers(project: Project, rewrites: AliasRewrite[]): void {
    const stale: string[] = [];
    for (const sourceFile of project.getSourceFiles()) {
        const imports = ts.preProcessFile(sourceFile.getFullText()).importedFiles;
        for (const imported of imports) {
            const rewrite = rewrites.find(candidate => matchesPrefix(imported.fileName, candidate.oldPrefix));
            if (rewrite) stale.push(`${sourceFile.getFilePath()}: ${imported.fileName}`);
        }
    }

    if (stale.length > 0) {
        throw new Error(`Stale TypeScript alias imports remain after move:\n${stale.join("\n")}`);
    }
}
