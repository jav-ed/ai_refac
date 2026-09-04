use anyhow::{Context, Result, bail};
use ra_ap_hir::Crate;
use ra_ap_ide::{Analysis, AnalysisHost, FileId, RootDatabase};
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace_at};
use ra_ap_project_model::CargoConfig;
use ra_ap_vfs::Vfs;
use std::path::{Path, PathBuf};

pub struct SemanticWorkspace {
    root: PathBuf,
    host: AnalysisHost,
    vfs: Vfs,
}

impl SemanticWorkspace {
    pub fn load(root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .with_context(|| format!("Could not resolve Cargo workspace at {}", root.display()))?;
        if root.to_str().is_none() {
            bail!(
                "Cargo workspace path is not valid UTF-8: {}",
                root.display()
            );
        }

        let load_config = LoadCargoConfig {
            load_out_dirs_from_check: false,
            with_proc_macro_server: ProcMacroServerChoice::None,
            prefill_caches: false,
            num_worker_threads: 1,
            proc_macro_processes: 1,
        };
        let (database, vfs, _) = load_workspace_at(
            &root,
            &CargoConfig::default(),
            &load_config,
            &|message| tracing::debug!(%message, "rust-analyzer workspace load"),
        )
        .with_context(|| format!("rust-analyzer could not load {}", root.display()))?;

        Ok(Self {
            root,
            host: AnalysisHost::with_database(database),
            vfs,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn database(&self) -> &RootDatabase {
        self.host.raw_database()
    }

    pub fn analysis(&self) -> Analysis {
        self.host.analysis()
    }

    pub fn file_path(&self, file_id: FileId) -> Result<PathBuf> {
        let path = self
            .vfs
            .file_path(file_id)
            .as_path()
            .context("rust-analyzer returned a virtual path for a workspace source file")?;
        Ok(PathBuf::from(path.as_os_str()))
    }

    pub fn is_local_crate(&self, krate: Crate) -> Result<bool> {
        let root_file = self.file_path(krate.root_file(self.database()))?;
        Ok(root_file.starts_with(&self.root))
    }

    pub fn file_belongs_to_crate(&self, file_id: FileId, krate: Crate) -> Result<bool> {
        let crates = self.analysis().crates_for(file_id)?;
        Ok(crates.contains(&krate.base()))
    }
}
