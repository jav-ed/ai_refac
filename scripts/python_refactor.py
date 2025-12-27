import sys
import os
import argparse
import json
from rope.base.project import Project
from rope.refactor.move import create_move
from rope.refactor.rename import Rename
from rope.base.fscommands import FileSystemCommands

def main():
    parser = argparse.ArgumentParser(description="Refactor Python files using Rope")
    subparsers = parser.add_subparsers(dest="command", help="Command to execute")

    # Move Command
    move_parser = subparsers.add_parser("move", help="Move a single file")
    move_parser.add_argument("source", help="Source file relative path")
    move_parser.add_argument("target", help="Target file relative path")
    move_parser.add_argument("--root", default=".", help="Project root directory")

    # Batch Command
    batch_parser = subparsers.add_parser("batch", help="Batch move files")
    batch_parser.add_argument("payload", help="JSON payload of file moves")
    batch_parser.add_argument("--root", default=".", help="Project root directory")

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        sys.exit(1)

    root_path = os.path.abspath(args.root)
    
    # Initialize Rope Project ONCE
    project = Project(root_path, fscommands=FileSystemCommands())
    project.prefs.set('save_objectdb', False)
    project.prefs.set('ignore_syntax_errors', True)

    try:
        if args.command == "move":
            perform_move(project, args.source, args.target)
        elif args.command == "batch":
            try:
                moves = json.loads(args.payload)
                # moves is expected to be a list of lists [[src, tgt], ...] or dicts
                count = 0
                for item in moves:
                    if isinstance(item, list) and len(item) >= 2:
                        src, tgt = item[0], item[1]
                    elif isinstance(item, dict):
                        src, tgt = item.get("source"), item.get("target")
                    else:
                        print(f"Skipping invalid item: {item}")
                        continue
                        
                    if src and tgt:
                        perform_move(project, src, tgt)
                        count += 1
                print(f"Batch operation completed. Refactored {count} files.")
            except json.JSONDecodeError:
                print("Error: Invalid JSON payload")
                sys.exit(1)
            except Exception as e:
                print(f"Batch processing error: {e}")
                sys.exit(1)

    finally:
        project.close()

def perform_move(project, source_rel, target_rel):
    try:
        resource = project.get_resource(source_rel)
    except Exception as e:
        print(f"Skipping {source_rel}: Not found in project ({e})")
        return

    src_dir = os.path.dirname(source_rel)
    tgt_dir = os.path.dirname(target_rel)
    src_name = os.path.basename(source_rel)
    tgt_name = os.path.basename(target_rel)

    # Ensure target directory exists for MOVE operations
    if src_dir != tgt_dir:
         full_tgt_dir = os.path.join(project.address, tgt_dir)
         if not os.path.exists(full_tgt_dir):
             os.makedirs(full_tgt_dir, exist_ok=True)

    print(f"Processing: {source_rel} -> {target_rel}")

    if src_dir == tgt_dir and src_name != tgt_name:
        # RENAME
        if tgt_name.endswith('.py'):
            new_name = tgt_name[:-3]
        else:
            new_name = tgt_name
            
        renamer = Rename(project, resource)
        changes = renamer.get_changes(new_name)
        project.do(changes)
        
    elif src_dir != tgt_dir:
         # MOVE operation
         tgt_folder = project.get_resource(tgt_dir)
         mover = create_move(project, resource)
         changes = mover.get_changes(tgt_folder)
         project.do(changes)
         
         # Check if we also need to RENAME (e.g. A.py -> B.py in new folder)
         if src_name != tgt_name:
             # Look for the moved file in the target directory
             new_loc_rel = os.path.join(tgt_dir, src_name)
             try:
                 moved_resource = project.get_resource(new_loc_rel)
                 renamer = Rename(project, moved_resource)
                 
                 # Derive new module/name (strip extension)
                 if tgt_name.endswith('.py'):
                     new_stem = tgt_name[:-3]
                 else:
                     new_stem = tgt_name
                     
                 changes = renamer.get_changes(new_stem)
                 project.do(changes)
             except Exception as e:
                 print(f"Error renaming after move: {e}")

    else:
        print(f"No op for {source_rel}")

if __name__ == "__main__":
    main()
