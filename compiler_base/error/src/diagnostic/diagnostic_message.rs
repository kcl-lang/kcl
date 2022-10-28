//! The crate provides `TemplateLoader` to load the diagnositc message displayed in diagnostics from "*.ftl" files,
//! `TemplateLoader` relies on 'fluent0.16.0' to support loading diagnositc message from "*.ftl" files.
//!
//! 'fluent0.16.0' is used to support diagnostic text template.
//! For more information about 'fluent0.16.0', see https://projectfluent.org/.

use anyhow::{bail, Context, Result};
use fluent::{FluentBundle, FluentResource};
use std::{fs, sync::Arc};
use unic_langid::langid;
use walkdir::{DirEntry, WalkDir};

use crate::diagnostic_handler::MessageArgs;
/// Struct `TemplateLoader` load template contents from "*.ftl" file.
/// `TemplateLoader` will operate on files locally.
pub(crate) struct TemplateLoader {
    template_inner: Arc<TemplateLoaderInner>,
}

impl TemplateLoader {
    /// Create an empty TemplateLoader that does not load any template files(*.ftl).
    pub(crate) fn default() -> Self {
        Self {
            template_inner: Arc::new(TemplateLoaderInner::default()),
        }
    }

    /// Create the `TemplateLoader` with template (*.ftl) files directory.
    /// `TemplateLoader` will load all the files end with "*.ftl" under the directory recursively.
    /// template_files
    ///      |
    ///      |---- template.ftl
    ///      |---- sub_template_files
    ///                  |
    ///                  |---- sub_template.ftl
    ///
    /// 'template.ftl' and 'sub_template.ftl' can both loaded by the `new_with_template_dir()`.
    pub(crate) fn new_with_template_dir(template_dir: &str) -> Result<Self> {
        let template_inner = TemplateLoaderInner::new_with_template_dir(template_dir)
            .with_context(|| format!("Failed to load '*.ftl' from '{}'", template_dir))?;
        Ok(Self {
            template_inner: Arc::new(template_inner),
        })
    }

    /// Get the message string from "*.ftl" file by `index`, `sub_index` and `MessageArgs`.
    /// For more information about "*.ftl" file, see the doc above `DiagnosticHandler`.
    /// "*.ftl" file looks like, e.g. './src/diagnostic/locales/en-US/default.ftl' :
    pub(crate) fn get_msg_to_str(
        &self,
        index: &str,
        sub_index: Option<&str>,
        args: &MessageArgs,
    ) -> Result<String> {
        let msg = match self.template_inner.get_template_bunder().get_message(index) {
            Some(m) => m,
            None => bail!("Message doesn't exist."),
        };

        let pattern = match sub_index {
            Some(s_id) => {
                let attr = msg.get_attribute(s_id).unwrap();
                attr.value()
            }
            None => match msg.value() {
                Some(v) => v,
                None => bail!("Message has no value."),
            },
        };

        let MessageArgs(args) = args;
        let value = self.template_inner.get_template_bunder().format_pattern(
            pattern,
            Some(args),
            &mut vec![],
        );
        Ok(value.to_string())
    }
}

/// `TemplateLoaderInner` is used to privatize the default constructor of `TemplateLoader`.
struct TemplateLoaderInner {
    template_bunder: FluentBundle<FluentResource>,
}

impl TemplateLoaderInner {
    fn default() -> Self {
        Self {
            template_bunder: FluentBundle::default(),
        }
    }

    fn new_with_template_dir(template_dir: &str) -> Result<Self> {
        let mut template_bunder = FluentBundle::new(vec![langid!("en-US")]);
        load_all_templates_in_dir_to_resources(template_dir, &mut template_bunder)
            .with_context(|| format!("Failed to load '*.ftl' from '{}'", template_dir))?;
        Ok(Self { template_bunder })
    }

    fn get_template_bunder(&self) -> &FluentBundle<FluentResource> {
        &self.template_bunder
    }
}

fn is_ftl_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".ftl"))
        .unwrap_or(false)
}

fn load_all_templates_in_dir_to_resources(
    dir: &str,
    fluent_bundle: &mut FluentBundle<FluentResource>,
) -> Result<()> {
    if !std::path::Path::new(&dir).exists() {
        bail!("Failed to load '*.ftl' dir");
    }

    for entry in WalkDir::new(dir) {
        let entry = entry?;

        if is_ftl_file(&entry) {
            let resource = fs::read_to_string(entry.path())?;

            match FluentResource::try_new(resource) {
                Ok(s) => {
                    if fluent_bundle.add_resource(s).is_err() {
                        bail!("Failed to parse an FTL string.")
                    }
                }
                Err(_) => bail!("Failed to add FTL resources to the bundle."),
            };
        }
    }
    Ok(())
}
