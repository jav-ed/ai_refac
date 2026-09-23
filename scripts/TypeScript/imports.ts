import { parseSync, Visitor, type Expression, type ImportAttribute, type ObjectExpression } from "oxc-parser";

export type ResolutionMode = "source" | "import" | "require";

export interface ModuleReference {
    value: string;
    start: number;
    end: number;
    mode: ResolutionMode;
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
    const parsed = parseSync(file, source);
    if (parsed.errors.length) {
        throw new Error(`Cannot parse ${file}:\n${parsed.errors.map(error => error.message).join("\n")}`);
    }
    const references: ModuleReference[] = [];
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
        references.push({ value, start: node.start, end: node.end, mode });
    }
    // Oxc nodes are scoped to one source file; only spans and decoded paths
    // survive this visit. Unrelated strings and comments are never candidates.
    new Visitor({
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
            if (node.callee.type === "Identifier" && node.callee.name === "require") {
                requireCall = true;
                const argument = node.arguments[0];
                if (argument?.type !== "SpreadElement") add(argument, "require");
            }
        },
    }).visit(parsed.program);
    if (boundRequire && requireCall) throw new Error(`Cannot safely resolve locally bound require calls in ${file}; lexical require wrappers need explicit handling.`);
    return references.sort((a, b) => a.start - b.start);
}

export function quotePath(value: string, quote: string): string {
    const escaped = JSON.stringify(value).slice(1, -1);
    if (quote === '"') return `"${escaped}"`;
    if (quote === "'") return `'${escaped.replaceAll("'", "\\'")}'`;
    if (quote === "`") return "`" + escaped.replaceAll("`", "\\`").replaceAll("${", "\\${") + "`";
    throw new Error(`Unsupported module literal delimiter ${JSON.stringify(quote)}`);
}
