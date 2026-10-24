import * as fs from "node:fs";
import * as path from "node:path";

export interface Move {
    source: string;
    target: string;
    directory: boolean;
    /** Another tool moves this path later; the plan fixes the code that uses it, the apply does not rename it. */
    external: boolean;
}
export interface MoveSet {
    moves: Move[];
    /** Every file the moves cover, with its target. */
    files: Map<string, string>;
    /** The files of the external moves only. */
    externalFiles: Map<string, string>;
}

export function within(file: string, directory: string): boolean {
    const relative = path.relative(directory, file);
    return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative));
}

/** Where `absolute` ends up: itself moved, or inside a folder that moves. Nothing when the moves do not touch it. */
export function targetOf(moves: MoveSet, absolute: string): string | undefined {
    const file = moves.files.get(absolute);
    if (file !== undefined) return file;
    const folder = moves.moves.find(move => move.directory && within(absolute, move.source));
    return folder && path.join(folder.target, path.relative(folder.source, absolute));
}

/** Only the moves another tool carries out. */
export function externalMoves(moves: MoveSet): MoveSet {
    return { moves: moves.moves.filter(move => move.external), files: moves.externalFiles, externalFiles: moves.externalFiles };
}

function listFiles(source: string): string[] {
    const stat = fs.lstatSync(source);
    if (stat.isSymbolicLink()) throw new Error(`Symlink moves are unsupported: ${source}`);
    if (stat.isFile()) return [source];
    if (!stat.isDirectory()) throw new Error(`Not a regular file or directory: ${source}`);
    return fs.readdirSync(source).flatMap(name => listFiles(path.join(source, name)));
}

function rejectSymlinkAncestors(root: string, file: string): void {
    for (let current = file; within(current, root); current = path.dirname(current)) {
        if (fs.lstatSync(current, { throwIfNoEntry: false })?.isSymbolicLink()) {
            throw new Error(`Move paths cannot traverse symlinks: ${current}`);
        }
        if (current === root) break;
    }
}

function parseMoves(root: string, input: unknown, external: boolean): Move[] {
    if (!Array.isArray(input)) throw new Error("Move batch must be a nonempty array of [source, target] pairs.");
    return input.map(item => {
        const pair = Array.isArray(item) ? item : [item?.source, item?.target];
        if (pair.length !== 2 || pair.some(value => typeof value !== "string" || !value)) {
            throw new Error(`Invalid move pair: ${JSON.stringify(item)}`);
        }
        const [source, target] = pair.map(value => path.resolve(root, value)) as [string, string];
        if (!within(source, root) || !within(target, root)) throw new Error(`Move must remain inside project ${root}: ${source} -> ${target}`);
        rejectSymlinkAncestors(root, source);
        rejectSymlinkAncestors(root, target);
        if (!fs.existsSync(source)) throw new Error(`Move source does not exist: ${source}`);
        if (fs.existsSync(target)) throw new Error(`Move target already exists: ${target}`);
        if (within(target, source)) throw new Error(`Cannot move a path into itself: ${source} -> ${target}`);
        return { source, target, directory: fs.statSync(source).isDirectory(), external };
    });
}

/**
 * `input` are the moves this engine carries out; `external` are moves another
 * tool carries out afterwards (the Markdown driver moves pages and pictures),
 * which the plan still has to know to fix the code that imports them.
 */
export function prepareMoves(root: string, input: unknown, external: unknown = []): MoveSet {
    const moves = [...parseMoves(root, input, false), ...parseMoves(root, external, true)];
    if (moves.length === 0) throw new Error("Move batch must be a nonempty array of [source, target] pairs.");
    for (let i = 0; i < moves.length; i++) {
        for (let j = i + 1; j < moves.length; j++) {
            const a = moves[i]!, b = moves[j]!;
            if ([a.source, a.target].some(left => [b.source, b.target].some(right => within(left, right) || within(right, left)))) {
                throw new Error(`Overlapping move requests are unsupported: ${a.source} and ${b.source}`);
            }
        }
    }
    const files = new Map<string, string>();
    const externalFiles = new Map<string, string>();
    for (const move of moves) {
        for (const file of listFiles(move.source)) {
            const target = move.directory ? path.join(move.target, path.relative(move.source, file)) : move.target;
            files.set(file, target);
            if (move.external) externalFiles.set(file, target);
        }
    }
    return { moves, files, externalFiles };
}
