import sys
import os
import argparse
from rope.base.project import Project
from rope.refactor.move import create_move
from rope.refactor.rename import Rename

def main():
    parser = argparse.ArgumentParser(description="Refactor Python files using Rope")
    parser.add_argument("source", help="Source file relative path")
    parser.add_argument("target", help="Target file relative path")
    parser.add_argument("--root", default=".", help="Project root directory")
    args = parser.parse_args()

    # Absolute paths
    root_path = os.path.abspath(args.root)
    
    # Initialize Rope Project with simple FileSystemCommands (no VCS)
    # This avoids issues if the project is not a git repo or if rope tries to invoke git
    from rope.base.fscommands import FileSystemCommands
    project = Project(root_path, fscommands=FileSystemCommands())
    project.prefs.set('save_objectdb', False)
    project.prefs.set('ignore_syntax_errors', True)
    
    # Get Resource
    try:
        resource = project.get_resource(args.source)
    except Exception as e:
        print(f"Error: Source file not found in project: {e}")
        sys.exit(1)

    src_dir = os.path.dirname(args.source)
    tgt_dir = os.path.dirname(args.target)
    src_name = os.path.basename(args.source)
    tgt_name = os.path.basename(args.target)

    # Perform Move or Rename
    try:
        if src_dir == tgt_dir and src_name != tgt_name:
            # RENAME
            # Rope adds extension automatically for python modules/files
            if tgt_name.endswith('.py'):
                new_name = tgt_name[:-3]
            else:
                new_name = tgt_name
                
            renamer = Rename(project, resource)
            changes = renamer.get_changes(new_name)
            # print(f"Changes: {changes.get_description()}")
            project.do(changes)
            print(f"Successfully renamed {args.source} to {args.target}")
        elif src_dir != tgt_dir:
             # MOVE
             # Assumption: Target path is the full file path. 
             # Rope Move requires a destination FOLDER.
             # If target filename differs, we strictly only support moving to a folder with same name for now
             # OR we strictly treat target as the folder if it exists?
             # RefactorRequest usually specifies full target file path.
             
             # If names differ, we can't easily do it in one atomic Rope move without rename.
             # MVP: Detect if target looks like a file path matches source name?
             
             if src_name != tgt_name:
                 print("Error: Changing filename during move (cross-directory rename) is not yet supported by this script.")
                 sys.exit(1)
            
             # Target directory resource
             tgt_folder = project.get_resource(tgt_dir)
             mover = create_move(project, resource)
             changes = mover.get_changes(tgt_folder)
             project.do(changes)
             print(f"Successfully moved {args.source} to {args.target}")
        else:
            print("Source and target are identical")
            
    except Exception as e:
        print(f"Error executing refactor: {e}")
        sys.exit(1)
    finally:
        project.close()

if __name__ == "__main__":
    main()
