//! Batch moves: several files in one `refac move`, per language and mixed.
//! Each language gets scenarios that verify not just file placement but that
//! the imports between the two moved files are updated by the driver.

mod dart;
mod failures;
mod go;
mod markdown;
mod python;
mod rust;
mod typescript;
