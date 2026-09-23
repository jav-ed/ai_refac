import * as path from "node:path";
import { ResolverFactory } from "oxc-resolver";
import ts from "typescript";
import type { ProjectConfig } from "./project";
import type { ResolutionMode } from "./imports";

export function makeResolver(project: ProjectConfig) {
    const cache = ts.createModuleResolutionCache(project.root, name => name, project.options);
    const assets = new ResolverFactory({
        tsconfig: project.configPath ? { configFile: project.configPath } : undefined,
        conditionNames: ["import", "default", ...project.options.customConditions ?? []],
        symlinks: !project.options.preserveSymlinks,
        nodePath: false,
    });
    return {
        resolve(file: string, specifier: string, requestedMode: ResolutionMode): string | undefined {
            const [request, suffix] = splitSuffix(specifier);
            // Asset requests have bundler filesystem semantics; TS/JS requests
            // always use the compiler's resolver, without constructing a Program.
            if (suffix || /\.(?:css|scss|sass|less|svg|png|jpe?g|webp|avif|gif|ico|woff2?|ttf|otf|wasm|txt|html|md|surql)$/i.test(request)) {
                return assets.sync(path.dirname(file), request).path;
            }
            const nodeMode = project.options.moduleResolution === ts.ModuleResolutionKind.Node16
                || project.options.moduleResolution === ts.ModuleResolutionKind.NodeNext;
            const mode = requestedMode === "require" ? ts.ModuleKind.CommonJS : requestedMode === "import" ? ts.ModuleKind.ESNext : nodeMode
                ? ts.getImpliedNodeFormatForFile(file, undefined, ts.sys, project.options)
                : ts.ModuleKind.ESNext;
            return ts.resolveModuleName(request, file, project.options, ts.sys, cache, undefined, mode).resolvedModule?.resolvedFileName;
        },
    };
}

export function splitSuffix(specifier: string): [string, string] {
    const index = specifier.search(/[?#]/);
    // A leading # is a package-imports key, not a URL fragment.
    const offset = index === 0 ? specifier.slice(1).search(/[?#]/) : index;
    const actual = index === 0 && offset >= 0 ? offset + 1 : offset;
    return actual < 0 ? [specifier, ""] : [specifier.slice(0, actual), specifier.slice(actual)];
}

function spelling(target: string, previous: string, options: ts.CompilerOptions): string {
    // moduleSuffixes inserts the platform suffix during resolution; carrying it
    // into the request would ask the compiler for e.g. b.native.native.ts.
    const sourceExtension = path.extname(target);
    const suffix = options.moduleSuffixes?.find(value => target.endsWith(value + sourceExtension));
    if (suffix) target = target.slice(0, -sourceExtension.length - suffix.length) + sourceExtension;
    const extension = path.extname(previous);
    if (extension === ".js" && /\.(?:tsx?|d\.ts)$/.test(target)) return target.replace(/\.(?:d\.ts|tsx?)$/, ".js");
    if (extension === ".jsx" && /\.tsx$/.test(target)) return target.replace(/\.tsx$/, ".jsx");
    if (extension === ".mjs" && /\.(?:d\.)?mts$/.test(target)) return target.replace(/\.(?:d\.)?mts$/, ".mjs");
    if (extension === ".cjs" && /\.(?:d\.)?cts$/.test(target)) return target.replace(/\.(?:d\.)?cts$/, ".cjs");
    if (!/\.(?:[cm]?[jt]sx?|json|css|svg|png|webp)$/.test(previous)) {
        return target.replace(/\.(?:d\.)?[cm]?[jt]sx?$/, "").replace(/\/index$/, "");
    }
    return target;
}

export function replacement(project: ProjectConfig, previous: string, importer: string, target: string, mode: ResolutionMode): string {
    const [specifier, suffix] = splitSuffix(previous);
    const nodeMode = project.options.moduleResolution === ts.ModuleResolutionKind.Node16 || project.options.moduleResolution === ts.ModuleResolutionKind.NodeNext;
    const esm = mode === "import" || (mode === "source" && ts.getImpliedNodeFormatForFile(importer, undefined, ts.sys, project.options) === ts.ModuleKind.ESNext);
    const explicitExtension = /\.(?:[cm]?[jt]sx?|json)$/.test(specifier);
    // Converting a bare package/alias to a relative Node ESM import needs the
    // emitted extension even though the original bare request had none.
    const spellingHint = nodeMode && esm && !explicitExtension
        ? /\.(?:d\.)?mts$/.test(target) ? ".mjs"
            : /\.(?:d\.)?cts$/.test(target) ? ".cjs"
                : target.endsWith(".tsx") && project.options.jsx === ts.JsxEmit.Preserve ? ".jsx" : ".js"
        : specifier;
    const spelled = spelling(target.replaceAll(path.sep, "/"), spellingHint, project.options);
    if (specifier.startsWith(".")) {
        let relative = path.relative(path.dirname(importer), spelled).replaceAll(path.sep, "/");
        if (!relative.startsWith(".")) relative = `./${relative}`;
        return relative + suffix;
    }
    const paths = project.options.paths ?? {};
    const base = project.options.baseUrl ?? (project.options as { pathsBasePath?: string }).pathsBasePath ?? project.root;
    for (const [alias, values] of Object.entries(paths)) {
        const [head, tail = ""] = alias.split("*");
        if (!specifier.startsWith(head!) || !specifier.endsWith(tail)) continue;
        for (const value of values) {
            const absolute = path.resolve(base, value).replaceAll(path.sep, "/");
            const [start, end = ""] = absolute.split("*");
            if (alias.includes("*") && absolute.includes("*") && spelled.startsWith(start!) && spelled.endsWith(end)) {
                return head + spelled.slice(start!.length, end ? -end.length : undefined) + tail + suffix;
            }
        }
    }
    // A fixed alias cannot name its moved target without changing tsconfig.
    // Use an explicit relative path, then require resolution verification.
    let relative = path.relative(path.dirname(importer), spelled).replaceAll(path.sep, "/");
    if (!relative.startsWith(".")) relative = `./${relative}`;
    return relative + suffix;
}
