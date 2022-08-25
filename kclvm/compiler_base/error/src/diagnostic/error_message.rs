//! The crate provides `TemplateLoader` to load the error message displayed in diagnostics from "*.ftl" files,
//!
use std::fs;

use anyhow::{bail, Context, Error, Result};
use fluent::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::langid;
use walkdir::{DirEntry, WalkDir};

/// Struct `TemplateLoader` load template contents from "*.ftl" file.
///
/// "*.ftl" file looks like, e.g. './src/diagnostic/locales/en-US/default.ftl' :
///
/// ``` ignore
/// 1.   invalid-syntax = Invalid syntax
/// 2.             .expected = Expected one of `{$expected_items}`
/// ```
///
/// - In line 1, `invalid-syntax` is a `index`, `Invalid syntax` is the `Message String` to this `index`.
/// - In line 2, `.expected` is another `index`, it is a `sub_index` of `invalid-syntax`.
/// - In line 2, `sub_index` must start with a point `.` and it is optional.
/// - In line 2, `Expected one of `{$expected_items}`` is the `Message String` to `.expected`. It is an interpolated string.
/// - In line 2, `{$expected_items}` is a `MessageArgs` of the `Expected one of `{$expected_items}``
/// and `MessageArgs` can be recognized as a Key-Value entry, it is optional.  
///
/// The pattern of above '*.ftl' file looks like:
/// ``` ignore
/// 1.   <'index'> = <'message_string' with optional 'MessageArgs'>
/// 2.             <optional 'sub_index' start with point> = <'message_string' with optional 'MessageArgs'>
/// ```
pub struct TemplateLoader {
    template_inner: TemplateLoaderInner,
}

impl TemplateLoader {
    /// Create the `TemplateLoader` with template (*.ftl) files directory.
    /// `TemplateLoader` will load all the files end with "*.ftl" under the directory recursively.
    ///
    /// template_files
    ///      |
    ///      |---- template.ftl
    ///      |---- sub_template_files
    ///                  |
    ///                  |---- sub_template.ftl
    ///
    /// 'template.ftl' and 'sub_template.ftl' can both loaded by the `new_with_template_dir()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
    /// let error_message = TemplateLoader::new_with_template_dir("./src/diagnostic/locales/en-US/");
    /// ```
    pub fn new_with_template_dir(template_dir: &str) -> Result<Self> {
        let template_inner = TemplateLoaderInner::new_with_template_dir(template_dir)
            .with_context(|| format!("Failed to load '*.ftl' from '{}'", template_dir))?;
        Ok(Self { template_inner })
    }

    /// Get the message string from "*.ftl" file by `index`, `sub_index` and `MessageArgs`.
    /// For more information about "*.ftl" file, see the doc above `TemplateLoader`.
    ///
    /// ``` ignore
    /// 1.   invalid-syntax = Invalid syntax
    /// 2.             .expected = Expected one of `{$expected_items}`
    /// ```
    /// And for the 'default.ftl' shown above, you can get messages as follow:
    ///
    /// 1. If you want the message 'Invalid syntax' in line 1.
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
    /// # use compiler_base_error::diagnostic::error_message::MessageArgs;
    /// # use std::borrow::Borrow;
    ///
    /// // 1. Prepare an empty `MessageArgs`, Message in line 1 is not an interpolated string.
    /// let no_args = MessageArgs::new();
    ///
    /// // 2. `index` is 'invalid-syntax' and has no `sub_index`.
    /// let index = "invalid-syntax";
    /// let sub_index = None;
    ///
    /// // 3. Create the `TemplateLoader` with template (*.ftl) files directory.
    /// let error_message = TemplateLoader::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    /// let msg_in_line_1 = error_message.get_msg_to_str(index, sub_index, &no_args).unwrap();
    ///
    /// assert_eq!(msg_in_line_1, "Invalid syntax");
    /// ```
    ///
    /// 2. If you want the message 'Expected one of `{$expected_items}`' in line 2.
    ///
    /// ```rust
    /// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
    /// # use compiler_base_error::diagnostic::error_message::MessageArgs;
    /// # use std::borrow::Borrow;
    ///
    /// // 1. Prepare the `MessageArgs` for `{$expected_items}`.
    /// let mut args = MessageArgs::new();
    /// args.set("expected_items", "I am an expected item");
    ///
    /// // 2. `index` is 'invalid-syntax'.
    /// let index = "invalid-syntax";
    ///
    /// // 3. `sub_index` is 'expected'.
    /// let sub_index = "expected";
    ///
    /// // 4. With the help of `TemplateLoader`, you can get the message in 'default.ftl'.
    /// let error_message = TemplateLoader::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
    /// let msg_in_line_2 = error_message.get_msg_to_str(index, Some(sub_index), &args).unwrap();
    ///
    /// assert_eq!(msg_in_line_2, "Expected one of `\u{2068}I am an expected item\u{2069}`");
    /// ```
    pub fn get_msg_to_str(
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
            Some(&args),
            &mut vec![],
        );
        Ok(value.to_string())
    }
}

/// `MessageArgs` is the arguments of the interpolated string.
///
/// `MessageArgs` is a Key-Value entry which only supports "set" and without "get".
/// You need getting nothing from `MessageArgs`. Only setting it and senting it to `TemplateLoader` is enough.
///
/// Note: Currently both `Key` and `Value` of `MessageArgs` types only support string (&str).
///
/// # Examples
///
/// ```rust
/// # use compiler_base_error::diagnostic::error_message::MessageArgs;
/// # use compiler_base_error::diagnostic::error_message::TemplateLoader;
/// # use std::borrow::Borrow;
///
/// let index = "invalid-syntax";
/// let sub_index = Some("expected");
/// let mut msg_args = MessageArgs::new();
/// // You only need "set()".
/// msg_args.set("This is Key", "This is Value");
///
/// // When you use it, just sent it to `TemplateLoader`.
/// let error_message = TemplateLoader::new_with_template_dir("./src/diagnostic/locales/en-US/").unwrap();
/// let msg_in_line_1 = error_message.get_msg_to_str(index, sub_index, &msg_args);
/// ```
///
/// For more information about the `TemplateLoader` see the doc above struct `TemplateLoader`.
pub struct MessageArgs<'a>(FluentArgs<'a>);
impl<'a> MessageArgs<'a> {
    pub fn new() -> Self {
        Self(FluentArgs::new())
    }

    pub fn set(&mut self, k: &'a str, v: &'a str) {
        self.0.set(k, v);
    }
}

// `TemplateLoaderInner` is used to privatize the default constructor of `TemplateLoader`.
struct TemplateLoaderInner {
    template_bunder: FluentBundle<FluentResource>,
}

impl TemplateLoaderInner {
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
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => bail!("Failed to load '*.ftl' dir"),
        };

        if is_ftl_file(&entry) {
            let resource = match fs::read_to_string(entry.path()) {
                Ok(res) => res,
                Err(_) => bail!("Failed to read '*ftl' file"),
            };

            let source = match FluentResource::try_new(resource) {
                Ok(s) => s,
                Err(_) => bail!("Failed to add FTL resources to the bundle."),
            };

            match fluent_bundle.add_resource(source) {
                Ok(_) => {}
                Err(_) => bail!("Failed to parse an FTL string."),
            }
        }
    }
    Ok(())
}
