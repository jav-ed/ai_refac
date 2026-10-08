use super::*;
use ra_ap_syntax::Edition;

fn imports(code: &str, name: &str) -> bool {
    imports_module_by_name(&SourceFile::parse(code, Edition::CURRENT).tree(), name)
}

#[test]
fn a_plain_import_of_the_module_counts() {
    assert!(imports("use crate::drivers::lsp_client;\n", "lsp_client"));
    assert!(imports("use super::lsp_client;\n", "lsp_client"));
}

#[test]
fn self_in_a_group_counts() {
    assert!(imports(
        "use crate::drivers::lsp_client::{self, Server};\n",
        "lsp_client"
    ));
}

#[test]
fn importing_an_item_of_the_module_or_renaming_it_does_not_count() {
    assert!(!imports(
        "use crate::drivers::lsp_client::Server;\n",
        "lsp_client"
    ));
    assert!(!imports(
        "use crate::drivers::lsp_client::{Server, Other};\n",
        "lsp_client"
    ));
    assert!(!imports(
        "use crate::drivers::lsp_client as client;\n",
        "lsp_client"
    ));
    assert!(!imports("fn main() {}\n", "lsp_client"));
}
