import sys
import os
import argparse
from rope.base.project import Project
from rope.refactor.move import MoveResource

def main():
    parser = argparse.ArgumentParser(description="Refactor Python files using Rope")
    parser.add_argument("source", help="Source file relative path")
    parser.add_argument("target", help="Target file relative path")
    parser.add_argument("--root", default=".", help="Project root directory")
    args = parser.parse_args()

    # Absolute paths
    root_path = os.path.abspath(args.root)
    
    # Initialize Rope Project
    project = Project(root_path)
    
    # Get Resource
    try:
        resource = project.get_resource(args.source)
    except Exception as e:
        print(f"Error: Source file not found in project: {e}")
        sys.exit(1)

    # Perform Move
    try:
        mover = MoveResource(project, resource)
        changes = mover.get_changes(args.target)
        project.do(changes)
        print(f"Successfully moved {args.source} to {args.target}")
    except Exception as e:
        print(f"Error executing move: {e}")
        sys.exit(1)
    finally:
        project.close()

if __name__ == "__main__":
    main()
