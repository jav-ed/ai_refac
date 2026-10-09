import * as fs from "node:fs";
import * as path from "node:path";
import ts from "typescript";
import { within, type MoveSet } from "./moves";

/**
 * The project's files as they will be once the moves are done, for the
 * resolver of a dry run: a moved file exists at its target and no longer at its
 * source, and reads as its source did. Every other path is the disk.
 */
export function afterMoves(moves: MoveSet): ts.ModuleResolutionHost {
    const sources = new Set(moves.files.keys());
    const originals = new Map<string, string>();
    for (const [source, target] of moves.files) originals.set(target, source);
    const movedDirectories = moves.moves.filter(move => move.directory).map(move => move.source);
    // A directory the moves create, and every folder above it.
    const created = new Set<string>();
    for (const target of originals.keys()) {
        for (let directory = path.dirname(target); !created.has(directory) && directory !== path.dirname(directory); directory = path.dirname(directory)) {
            created.add(directory);
        }
    }
    const stat = (file: string) => fs.statSync(file, { throwIfNoEntry: false });
    return {
        fileExists: file => originals.has(file) || (!sources.has(file) && (stat(file)?.isFile() ?? false)),
        readFile: file => {
            const original = originals.get(file) ?? (sources.has(file) ? undefined : file);
            return original !== undefined && stat(original)?.isFile() ? fs.readFileSync(original, "utf8") : undefined;
        },
        directoryExists: directory => created.has(directory) || ((stat(directory)?.isDirectory() ?? false) && !movedDirectories.some(source => within(directory, source))),
        realpath: file => (originals.has(file) || !fs.existsSync(file) ? file : fs.realpathSync.native(file)),
        getCurrentDirectory: () => process.cwd(),
        getDirectories: directory => ts.sys.getDirectories(directory),
        useCaseSensitiveFileNames: ts.sys.useCaseSensitiveFileNames,
    } as ts.ModuleResolutionHost;
}
