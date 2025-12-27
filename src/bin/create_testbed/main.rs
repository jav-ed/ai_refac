mod utils;
mod typescript;
mod python;
mod rust;
mod go;

use std::fs;

fn main() -> std::io::Result<()> {
    // 1. Resolve Paths
    let current_dir = std::env::current_dir()?;
    let trials_dir = current_dir.join("Trials");
    let refac_tree_dir = trials_dir.join("0_Refac_Tree");

    println!("=== Modular Testbed Generator ===");
    println!("Target Directory: {:?}", refac_tree_dir);

    // 2. Clean Slate
    if refac_tree_dir.exists() {
        println!("Cleaning existing directory...");
        fs::remove_dir_all(&refac_tree_dir)?;
    }
    fs::create_dir_all(&refac_tree_dir)?;

    // 3. Generate Languages
    typescript::generate(&refac_tree_dir)?;
    python::generate(&refac_tree_dir)?;
    rust::generate(&refac_tree_dir)?;
    go::generate(&refac_tree_dir)?;

    println!("\n✅ Complex Testbed successfully created at:");
    println!("{}", refac_tree_dir.display());
    Ok(())
}
