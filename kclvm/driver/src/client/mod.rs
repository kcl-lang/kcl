mod fs;
mod git;
mod oci;

use anyhow::Result;
use indexmap::IndexSet;
use kclvm_config::modfile::{
    get_vendor_home, load_mod_file, load_mod_lock_file, LockDependency, ModLockFile,
};
use kclvm_config::modfile::{Dependency, GitSource, ModFile, OciSource};
use kclvm_utils::fslock::open_lock_file;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference, RegistryOperation};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::toolchain::{Metadata, Package};

use self::git::clone_git_repo_to;

pub const DEFAULT_OCI_REGISTRY: &str = "ghcr.io/kcl-lang";
pub const KCL_SRC_URL_ENV_VAR: &str = "KCL_SRC_URL";
pub const KCL_SRC_URL_USERNAME_ENV_VAR: &str = "KCL_SRC_USERNAME";
pub const KCL_SRC_URL_PASSWORD_ENV_VAR: &str = "KCL_SRC_PASSWORD";

#[derive(Default)]
pub struct ModClient {
    /// The mod file config of current module.
    mod_file: ModFile,
    /// The mod lock file config of current module.
    mod_lock_file: Option<ModLockFile>,
    /// The package search work directory.
    work_dir: PathBuf,
    /// Optional vendor home.
    vendor: Option<PathBuf>,
    /// A lazy OCI client.
    oci_client: Arc<Client>,
}

unsafe impl Send for ModClient {}
unsafe impl Sync for ModClient {}

impl ModClient {
    /// New a default mod client to fetch metadata ot update dependencies.
    #[inline]
    pub fn new<P: AsRef<Path>>(work_dir: P) -> Result<Self> {
        Self::new_with_oci_client(work_dir, Arc::new(Client::default()))
    }

    /// New a default mod client to fetch metadata ot update dependencies.
    pub fn new_with_oci_client<P: AsRef<Path>>(
        work_dir: P,
        oci_client: Arc<Client>,
    ) -> Result<Self> {
        Ok(Self {
            work_dir: work_dir.as_ref().to_path_buf(),
            mod_file: load_mod_file(&work_dir)?,
            mod_lock_file: load_mod_lock_file(&work_dir).ok(),
            vendor: None,
            oci_client,
        })
    }

    /// Auth the oci client
    pub fn auth(&self) -> Result<()> {
        if let (Ok(username), Ok(password)) = (
            std::env::var(KCL_SRC_URL_USERNAME_ENV_VAR),
            std::env::var(KCL_SRC_URL_PASSWORD_ENV_VAR),
        ) {
            let rt = tokio::runtime::Runtime::new()?;
            let image: Reference = self.default_oci_registry().parse()?;
            let auth = RegistryAuth::Basic(username, password);
            let _ = rt.block_on(async {
                self.oci_client
                    .auth(&image, &auth, RegistryOperation::Pull)
                    .await
                    .ok()?
            });
        }
        Ok(())
    }

    #[inline]
    pub fn default_oci_registry(&self) -> String {
        std::env::var(KCL_SRC_URL_ENV_VAR).unwrap_or(DEFAULT_OCI_REGISTRY.to_string())
    }

    /// Change the work directory.
    pub fn change_work_dir<P: AsRef<Path>>(&mut self, work_dir: P) -> Result<()> {
        let work_dir = work_dir.as_ref().to_path_buf();
        self.mod_file = load_mod_file(&work_dir)?;
        if let Ok(mod_lock_file) = load_mod_lock_file(&work_dir) {
            self.mod_lock_file = Some(mod_lock_file);
        }
        self.work_dir = work_dir;
        Ok(())
    }

    /// Set the vendor path.
    pub fn set_vendor<P: AsRef<Path>>(&mut self, vendor: P) -> &mut Self {
        let vendor = vendor.as_ref().to_path_buf();
        self.vendor = Some(vendor);
        self
    }

    /// Lock the kcl.mod file and resolve package deps to metadata, note this function will download
    /// deps from remote sources. If the dependency is downloaded to the local path, calculate the
    /// package metadata.
    pub fn lock_and_resolve_all_deps<P: AsRef<Path>>(
        &mut self,
        lock_file: P,
        update: bool,
    ) -> Result<Metadata> {
        let mut lock_guard =
            open_lock_file(lock_file.as_ref().to_string_lossy().to_string().as_str())?;
        lock_guard.lock()?;
        self.resolve_all_deps(update)
    }

    /// Resolve package deps to metadata, note this function will download deps from remote sources.
    /// If the dependency is downloaded to the local path, calculate the package metadata.
    pub fn resolve_all_deps(&mut self, update: bool) -> Result<Metadata> {
        let mut metadata = Metadata::default();
        match &self.mod_file.dependencies {
            Some(dependencies) if !dependencies.is_empty() => {
                let vendor = self.get_vendor_path()?;
                let mut paths: IndexSet<PathBuf> = IndexSet::default();
                for (name, dep) in dependencies {
                    let path = if update {
                        let path = self.download_dep_to_vendor(name, dep, &vendor)?;
                        paths.insert(path.clone());
                        path
                    } else {
                        Default::default()
                    };
                    metadata.packages.insert(
                        name.replace('-', "_"),
                        Package {
                            name: name.to_string(),
                            manifest_path: path,
                        },
                    );
                }
                for path in paths {
                    if let Ok(mut client) =
                        ModClient::new_with_oci_client(path, self.oci_client.clone())
                    {
                        let new_metadata = client.resolve_all_deps(update)?;
                        for (name, package) in new_metadata.packages {
                            metadata.packages.entry(name).or_insert(package);
                        }
                    }
                }
                Ok(metadata)
            }
            _ => Ok(metadata),
        }
    }

    /// Download a dependency to the local path.
    pub fn download_dep_to_vendor(
        &self,
        name: &str,
        dep: &Dependency,
        vendor: &Path,
    ) -> Result<PathBuf> {
        let path = self.get_local_path_from_dep(name, dep);
        let path = Path::new(vendor).join(path);
        match dep {
            Dependency::Version(version) => self.download_oci_source_to(
                name,
                &OciSource {
                    oci: oci::oci_reg_repo_join(&self.default_oci_registry(), name),
                    tag: Some(version.to_string()),
                },
                &path,
            ),
            Dependency::Git(git_source) => self.download_git_source_to(git_source, &path),
            Dependency::Oci(oci_source) => self.download_oci_source_to(name, oci_source, &path),
            Dependency::Local(_) => {
                // Nothing to do for the local source.
                Ok(path)
            }
        }
    }

    /// Get the vendor path.
    pub fn get_vendor_path(&self) -> Result<PathBuf> {
        Ok(match &self.vendor {
            Some(vendor) => {
                std::fs::create_dir_all(vendor)?;
                vendor.to_path_buf()
            }
            None => PathBuf::from(get_vendor_home()),
        })
    }

    pub fn download_git_source_to(&self, git_source: &GitSource, path: &Path) -> Result<PathBuf> {
        let path = clone_git_repo_to(
            &git_source.git,
            &git_source.branch,
            &git_source.tag,
            &git_source.commit,
            path,
        )?;
        Ok(path)
    }

    pub fn download_oci_source_to(
        &self,
        name: &str,
        oci_source: &OciSource,
        path: &Path,
    ) -> Result<PathBuf> {
        let rt = tokio::runtime::Runtime::new()?;
        let path = rt.block_on(async {
            oci::pull_oci_and_extract_layer(
                &self.oci_client,
                name,
                &oci_source.oci,
                &oci_source.tag,
                path,
            )
            .await
        })?;
        Ok(path)
    }

    /// Get the dependency store path
    pub fn get_local_path_from_dep(&self, name: &str, dep: &Dependency) -> String {
        match dep {
            Dependency::Version(version) => {
                format!("{}_{}", name, version)
            }
            Dependency::Git(git_source) => {
                if let Some(tag) = &git_source.tag {
                    format!("{}_{}", name, tag)
                } else if let Some(commit) = &git_source.commit {
                    format!("{}_{}", name, commit)
                } else if let Some(branch) = &git_source.branch {
                    format!("{}_{}", name, branch)
                } else {
                    format!("{name}_latest")
                }
            }
            // Just returns the folder.
            Dependency::Oci(_) => "".to_string(),
            Dependency::Local(local_source) => {
                let local_path = PathBuf::from(&local_source.path);
                if local_path.is_absolute() {
                    local_source.path.clone()
                } else {
                    self.work_dir
                        .join(&local_source.path)
                        .to_string_lossy()
                        .to_string()
                }
            }
        }
    }

    /// Get the lock dependency store path
    pub fn get_local_path_from_lock_dep(&self, lock_dep: &LockDependency) -> Option<String> {
        if lock_dep.reg.is_some() {
            lock_dep.full_name.clone()
        } else if let Some(git_url) = &lock_dep.url {
            Some(self.get_local_path_from_dep(
                &lock_dep.name,
                &Dependency::Git(GitSource {
                    git: git_url.to_string(),
                    branch: lock_dep.branch.clone(),
                    commit: lock_dep.commit.clone(),
                    tag: lock_dep.git_tag.clone(),
                    version: lock_dep.version.clone(),
                }),
            ))
        } else {
            match &self.mod_file.dependencies {
                Some(dependencies) => dependencies
                    .get(&lock_dep.name)
                    .as_ref()
                    .map(|dep| self.get_local_path_from_dep(&lock_dep.name, dep)),
                None => None,
            }
        }
    }

    /// Get the package metadata from the kcl.mod.lock file.
    pub fn get_metadata_from_mod_lock_file(&self) -> Option<Metadata> {
        if let Some(mod_lock_file) = &self.mod_lock_file {
            if let Some(dependencies) = &mod_lock_file.dependencies {
                let vendor = self.get_vendor_path().ok()?;
                let mut metadata = Metadata::default();
                for (name, dep) in dependencies {
                    metadata.packages.insert(
                        name.replace('-', "_").to_string(),
                        Package {
                            name: name.to_string(),
                            manifest_path: match self.get_local_path_from_lock_dep(dep) {
                                Some(path) => vendor.join(path),
                                None => "".into(),
                            },
                        },
                    );
                }
                return Some(metadata);
            }
        }
        None
    }
}
