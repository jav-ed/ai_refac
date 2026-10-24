/** A file path written in a stylesheet. `start`..`end` is the path alone, without its quotes. */
export interface CssReference { value: string; start: number; end: number; quote: string; }

const IDENTIFIER_CHARACTER = /[\w-]/;

/**
 * The local file paths a stylesheet names: `url(...)` with or without quotes
 * and `@import "..."`. Comments and strings that are not such an argument are
 * skipped, so a path that is only mentioned is not a reference.
 *
 * Addresses, site-root paths, `#fragments`, `data:` URIs and anything built by
 * a preprocessor (`$var`, `#{...}`, `@{...}`, `var(...)`) are not paths of this
 * file and are left out. A path with a CSS escape is an error: it cannot be
 * edited without decoding it, and skipping it would leave it behind unnoticed.
 */
export function cssReferences(text: string): CssReference[] {
    const found: CssReference[] = [];
    function add(value: string, start: number, quote: string) {
        if (!isLocalPath(value)) return;
        if (value.includes("\\")) throw new Error(`Cannot move a CSS path with an escape: ${JSON.stringify(value)}`);
        found.push({ value, start, end: start + value.length, quote });
    }
    let i = 0;
    while (i < text.length) {
        if (text.startsWith("/*", i)) {
            const end = text.indexOf("*/", i + 2);
            i = end < 0 ? text.length : end + 2;
        } else if (text[i] === '"' || text[i] === "'") {
            i = endOfString(text, i);
        } else if (text.slice(i, i + 4).toLowerCase() === "url(" && !(i > 0 && IDENTIFIER_CHARACTER.test(text[i - 1]!))) {
            i = readArgument(text, i + 4, add);
        } else if (text.slice(i, i + 7).toLowerCase() === "@import") {
            i = readImport(text, i + 7, add);
        } else {
            i++;
        }
    }
    return found;
}

type Add = (value: string, start: number, quote: string) => void;

/** After `url(`: a quoted or an unquoted path. Returns where to go on. */
function readArgument(text: string, from: number, add: Add): number {
    let i = from;
    while (/\s/.test(text[i] ?? "")) i++;
    const quote = text[i];
    if (quote === '"' || quote === "'") {
        const end = endOfString(text, i);
        add(text.slice(i + 1, end - 1), i + 1, quote);
        return end;
    }
    let end = i;
    while (end < text.length && text[end] !== ")" && !/\s/.test(text[end]!)) end++;
    add(text.slice(i, end), i, "");
    return end;
}

/** After `@import`: a quoted path (a `url(...)` is found by the main loop). */
function readImport(text: string, from: number, add: Add): number {
    let i = from;
    while (/\s/.test(text[i] ?? "")) i++;
    const quote = text[i];
    if (quote !== '"' && quote !== "'") return i;
    const end = endOfString(text, i);
    add(text.slice(i + 1, end - 1), i + 1, quote);
    return end;
}

/** The index after the string that starts at `from`; a backslash protects the next character. */
function endOfString(text: string, from: number): number {
    const quote = text[from]!;
    for (let i = from + 1; i < text.length; i++) {
        if (text[i] === "\\") i++;
        else if (text[i] === quote) return i + 1;
    }
    return text.length;
}

function isLocalPath(value: string): boolean {
    if (!value || /^(?:[a-z][a-z0-9+.-]*:|\/|#)/i.test(value)) return false;
    return !/[$]|#\{|@\{|var\(|\{|\}/.test(value);
}
