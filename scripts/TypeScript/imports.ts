import { Visitor, type CallExpression, type Expression, type ImportAttribute, type ObjectExpression } from "oxc-parser";
import { cssReferences } from "./Embedded/css";
import { partsOf } from "./Embedded/regions";
import { looksLikeRootPath, type PathString } from "./strings";

export type ResolutionMode = "source" | "import" | "require";

export interface ModuleReference {
    value: string;
    start: number;
    end: number;
    mode: ResolutionMode;
    /**
     * Set for a path in a stylesheet. Its span is the path alone (no quotes), `quote`
     * is the one around it (empty for `url(path)`), and a path without a leading `.`
     * is relative to the file too.
     */
    css?: { quote: string };
    /** Set for a specifier written as an attribute value of the template (`<script src="...">`); the span includes its quotes. */
    html?: true;
}

function attributeMode(attributes: ImportAttribute[]): ResolutionMode {
    const attribute = attributes.find(item => (item.key.type === "Identifier" ? item.key.name : item.key.value) === "resolution-mode");
    if (!attribute) return "source";
    if (attribute.value.value !== "import" && attribute.value.value !== "require") throw new Error("Invalid resolution-mode import attribute");
    return attribute.value.value;
}

function typeMode(options: ObjectExpression | null): ResolutionMode {
    if (!options) return "source";
    for (const property of options.properties) {
        if (property.type !== "Property" || property.value.type !== "ObjectExpression") continue;
        for (const entry of property.value.properties) {
            if (entry.type !== "Property") continue;
            const key = entry.key.type === "Identifier" ? entry.key.name : entry.key.type === "Literal" ? entry.key.value : undefined;
            if (key !== "resolution-mode") continue;
            if (entry.value.type !== "Literal" || (entry.value.value !== "import" && entry.value.value !== "require")) throw new Error("Invalid resolution-mode type import attribute");
            return entry.value.value;
        }
    }
    return "source";
}

export function collectImports(file: string, source: string): ModuleReference[] {
    return scanFile(file, source).modules;
}

/**
 * What a file names: its module specifiers, the patterns of its `import.meta.glob` and
 * `Astro.glob` calls, the paths relative to the file that its frontmatter holds and, apart
 * from those, the strings of its code that may be root paths.
 */
export function scanFile(file: string, source: string): { modules: ModuleReference[]; strings: PathString[]; globs: PathString[]; paths: PathString[] } {
    const references: ModuleReference[] = [];
    const strings: PathString[] = [];
    const globs: PathString[] = [];
    // Offsets of the region being visited, turned into offsets of the whole file.
    let shift = 0;
    let boundRequire = false;
    let requireCall = false;
    // A locally bound require may be an arbitrary function or use another base
    // directory. Reject ambiguity instead of rewriting an unrelated string.
    function binding(node: unknown): void {
        if (!node || typeof node !== "object") return;
        const value = node as Record<string, unknown>;
        if (value.type === "Identifier" && value.name === "require") boundRequire = true;
        if (value.type === "RestElement") binding(value.argument);
        if (value.type === "AssignmentPattern") binding(value.left);
        if (value.type === "TSParameterProperty") binding(value.parameter);
        if (value.type === "ArrayPattern") for (const item of value.elements as unknown[]) binding(item);
        if (value.type === "ObjectPattern") for (const item of value.properties as Record<string, unknown>[]) binding(item.type === "RestElement" ? item.argument : item.value);
    }
    function add(node: Expression | null | undefined, mode: ResolutionMode = "source") {
        if (!node) return;
        let value: string;
        if (node.type === "Literal" && typeof node.value === "string") value = node.value;
        else if (node.type === "TemplateLiteral" && node.expressions.length === 0) value = node.quasis[0]!.value.cooked!;
        else return; // Computed module paths have no statically knowable target.
        references.push({ value, start: node.start + shift, end: node.end + shift, mode });
    }
    function candidate(node: Expression) {
        const value = node.type === "Literal" && typeof node.value === "string" ? node.value
            : node.type === "TemplateLiteral" && node.expressions.length === 0 ? node.quasis[0]!.value.cooked
            : undefined;
        if (value && looksLikeRootPath(value)) strings.push({ value, start: node.start + shift, end: node.end + shift });
    }
    // `import.meta.glob(patterns, options)` and `Astro.glob(patterns)`. With a `base` option the
    // patterns are relative to that folder, which is not read here, so they are left alone.
    function globCall(node: CallExpression) {
        const callee = node.callee;
        const isGlob = callee.type === "MemberExpression" && !callee.computed && callee.property.type === "Identifier" && callee.property.name === "glob"
            && (callee.object.type === "MetaProperty" || (callee.object.type === "Identifier" && callee.object.name === "Astro"));
        if (!isGlob) return;
        const [patterns, options] = node.arguments;
        if (options?.type === "ObjectExpression" && options.properties.some(property => property.type === "Property" && (property.key.type === "Identifier" ? property.key.name : property.key.type === "Literal" ? property.key.value : undefined) === "base")) return;
        for (const item of patterns?.type === "ArrayExpression" ? patterns.elements : [patterns]) {
            if (!item || item.type === "SpreadElement") continue;
            const value = item.type === "Literal" && typeof item.value === "string" ? item.value
                : item.type === "TemplateLiteral" && item.expressions.length === 0 ? item.quasis[0]!.value.cooked
                : undefined;
            if (value) globs.push({ value, start: item.start + shift, end: item.end + shift });
        }
    }
    // Oxc nodes are scoped to one source file; only spans and decoded paths
    // survive this visit. Comments are never candidates.
    const visitor = new Visitor({
        Literal(node) { candidate(node as Expression); },
        TemplateLiteral(node) { candidate(node as Expression); },
        ImportDeclaration(node) { add(node.source, attributeMode(node.attributes)); },
        ExportNamedDeclaration(node) { add(node.source, attributeMode(node.attributes)); },
        ExportAllDeclaration(node) { add(node.source, attributeMode(node.attributes)); },
        ImportExpression(node) { add(node.source, "import"); },
        TSImportType(node) { add(node.source, typeMode(node.options)); },
        TSExternalModuleReference(node) { add(node.expression, "require"); },
        TSModuleDeclaration(node) { if (node.id.type === "Literal") add(node.id); },
        VariableDeclarator(node) { binding(node.id); },
        AssignmentExpression(node) { binding(node.left); },
        FunctionDeclaration(node) { binding(node.id); node.params.forEach(binding); },
        FunctionExpression(node) { binding(node.id); node.params.forEach(binding); },
        ArrowFunctionExpression(node) { node.params.forEach(binding); },
        ClassDeclaration(node) { binding(node.id); },
        ClassExpression(node) { binding(node.id); },
        CatchClause(node) { binding(node.param); },
        ImportSpecifier(node) { binding(node.local); },
        ImportDefaultSpecifier(node) { binding(node.local); },
        ImportNamespaceSpecifier(node) { binding(node.local); },
        TSImportEqualsDeclaration(node) { binding(node.id); },
        CallExpression(node) {
            globCall(node);
            if (node.callee.type === "Identifier" && node.callee.name === "require") {
                requireCall = true;
                const argument = node.arguments[0];
                if (argument?.type !== "SpreadElement") add(argument, "require");
            }
        },
    });
    const parts = partsOf(file, source);
    for (const region of parts.code) {
        shift = region.shift;
        visitor.visit(region.program);
    }
    for (const style of parts.styles) {
        for (const reference of cssReferences(style.text)) {
            references.push({ value: reference.value, start: reference.start + style.shift, end: reference.end + style.shift, mode: "source", css: { quote: reference.quote } });
        }
    }
    for (const specifier of parts.specifiers) {
        references.push({ ...specifier, mode: "source", html: true });
    }
    if (boundRequire && requireCall) throw new Error(`Cannot safely resolve locally bound require calls in ${file}; lexical require wrappers need explicit handling.`);
    // An import's own string is a module, not a root path.
    const taken = new Set([...references, ...globs].map(reference => reference.start));
    return { modules: references.sort((a, b) => a.start - b.start), strings: strings.filter(item => !taken.has(item.start)), globs, paths: parts.paths };
}

/** A path written inside the quotes of a stylesheet (or without any, in `url(...)`), without the quotes. */
export function quoteCssPath(value: string, quote: string): string {
    if (quote === "") {
        if (/[\s"'()\\]/.test(value)) throw new Error(`Cannot write ${JSON.stringify(value)} in an unquoted url(): it has a space, quote, parenthesis or backslash. Quote it in the stylesheet first.`);
        return value;
    }
    return value.replaceAll("\\", "\\\\").replaceAll(quote, `\\${quote}`);
}

/** A specifier inside the quotes of an HTML attribute: no escapes exist there, so a character that would need one is refused. */
export function quoteAttribute(value: string, quote: string): string {
    if (value.includes(quote) || /[&<>\\]/.test(value)) throw new Error(`Cannot write ${JSON.stringify(value)} in an HTML attribute: it has a quote, &, <, > or a backslash.`);
    return `${quote}${value}${quote}`;
}

export function quotePath(value: string, quote: string): string {
    const escaped = JSON.stringify(value).slice(1, -1);
    if (quote === '"') return `"${escaped}"`;
    if (quote === "'") return `'${escaped.replaceAll("'", "\\'")}'`;
    if (quote === "`") return "`" + escaped.replaceAll("`", "\\`").replaceAll("${", "\\${") + "`";
    throw new Error(`Unsupported module literal delimiter ${JSON.stringify(quote)}`);
}
