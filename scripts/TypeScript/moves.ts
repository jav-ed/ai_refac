import * as fs from "node:fs";
import * as path from "node:path";

export interface Move { source: string; target: string; directory: boolean; }
export interface MoveSet { moves: Move[]; files: Map<string, string>; }

export function within(file: string, directory: string): boolean {
    const relative = path.relative(directory, file);
    return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative));
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

export function prepareMoves(root: string, input: unknown): MoveSet {
    if (!Array.isArray(input) || input.length === 0) throw new Error("Move batch must be a nonempty array of [source, target] pairs.");
    const moves: Move[] = input.map(item => {
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
        return { source, target, directory: fs.statSync(source).isDirectory() };
    });
    for (let i = 0; i < moves.length; i++) {
        for (let j = i + 1; j < moves.length; j++) {
            const a = moves[i]!, b = moves[j]!;
            if ([a.source, a.target].some(left => [b.source, b.target].some(right => within(left, right) || within(right, left)))) {
                throw new Error(`Overlapping move requests are unsupported: ${a.source} and ${b.source}`);
            }
        }
    }
    const files = new Map<string, string>();
    for (const move of moves) {
        for (const file of listFiles(move.source)) {
            files.set(file, move.directory ? path.join(move.target, path.relative(move.source, file)) : move.target);
        }
    }
    return { moves, files };
}
