# PkgPath: typed boundary for compiler-side path unification

| Status | Draft (architecture track, incremental) |
| --- | --- |
| Related | #966, #2178, PR #2179 |
| Author | follow-up to zong-zhe's 2024 WIP design ([comment](https://github.com/kcl-lang/kcl/issues/966#issuecomment-1987663960)) and Peefy's 2026 status note |

## Summary

Introduce a `PkgPath` newtype around the dotted pkgpath string (`a.b.c`)
so that the conversion to and from `PathBuf` lives in exactly one place.
All internal compiler code carries `PkgPath`; only the boundary layers
(parser entry, toolchain, LSP `didOpen`/`didChange` handlers, bundle
rewriter) deal with raw `PathBuf` / `&str`. The goal is to make the class
of Windows-only failures we've fixed repeatedly — hardcoded `/`
separators, mismatched `ends_with("/...")` / `trim_end_matches("/...")`
suffixes, dropped `let pkgpath = ` match bindings — impossible to write in
the first place.

This is the incremental shape of the VFS work that issue #966 proposed
in 2024. The full `VFS` trait + `SourceFile` rewrite in zong-zhe's
original design is **explicitly deferred** to a separate document; this
doc covers the typed-key step that all later work can build on.

## Status

Done in #2178 and PR #2179:

- 5 sites that hardcoded `/` between pkgpath segments now use
  `pkgpath_to_rel_path_buf` / `pkgpath_to_path_buf`.
- `get_pkg_list` no longer drops the `let pkgpath = match pkgpath.chars().next()`
  binding that turns `./...` into a cwd-anchored walk and `a.b.c` into a
  real directory path.
- LSP `external_pkg_real_path` and driver `get_real_path_from_external`
  both go through a single `external_pkgpath_to_rel_path_buf` helper.
- `test_get_pkg_list` actually verifies the binding fix (used to only
  assert on `.len()`).

Open after this PR:

- Sites still take `&str` and `PathBuf` interchangeably. A `PkgPath`
  newtype would let the type system catch a regression instead of a test.
- The full VFS abstraction from the 2024 design remains on the shelf.

## Why now

The compiler-side work shipped so far has been a sequence of targeted
fixes against concrete failures. Each fix was correct in isolation, but
the underlying problem is structural: pkgpath (a `String`) and
filesystem path (a `PathBuf`) flow through the same call sites and the
caller has to remember which is which and how to convert. Helper
functions help, but they don't stop the next contributor from writing
`Path::new(root).join(pkgpath.replace('.', "/"))` again — there's
nothing in the type system to catch it.

A `PkgPath` newtype is the smallest change that closes the gap. It
doesn't replace the filesystem layer, doesn't change the public API,
and doesn't break any golden file. It just makes the conversion
monomorphic: you can have a `PkgPath` (in-memory) or a `PathBuf` (on
disk) and converting between them is one `From` impl.

## Non-goals

- **Full VFS rewrite.** The 2024 WIP `VFS` trait, the `rustc::SourceFile`
  wrapper, and the loader/Salsa/VFS split described in zong-zhe's
  comment are out of scope. They require an architectural decision the
  maintainer explicitly deferred in 2026.
- **Per-crate path conversion helpers.** `pkgpath_to_path_buf` and
  friends are not going away; the newtype is layered on top of them.
- **Bundle / LSP URI display normalization.** `crates/tools/src/testing/
  mod.rs::normalize_coverage_key` and the LSP `find_refs` /
  `goto_def` `replace("\\", "/")` calls are OS-path-to-display
  normalization, not pkgpath conversion. Different problem; leave alone.
- **Changing the grammar or the public `kcl` CLI.** Internal-only.

## Proposed design

### The type

```rust
// crates/utils/src/pkgpath.rs (already hosts the conversion helpers)

/// A dotted pkgpath (e.g. `a.b.c`), the compiler's canonical in-memory
/// key for a package or module.
///
/// Use `PkgPath::to_path(&root)` to resolve to a filesystem path. Use
/// `PkgPath::parse(&str)` to parse user input (relative imports,
/// `kcl.mod` entries, CLI `-O key=value` arguments).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PkgPath {
    pkgpath: String,
}

impl PkgPath {
    pub fn new(pkgpath: impl Into<String>) -> Self { ... }
    pub fn as_str(&self) -> &str { &self.pkgpath }
    pub fn is_absolute(&self) -> bool { !self.pkgpath.starts_with('.') }

    /// Resolve to an absolute filesystem path under `root` using the
    /// platform-correct separator. This is the single place where the
    /// `.` → `/` (or `\`) mapping lives.
    pub fn to_path(&self, root: &Path) -> PathBuf {
        pkgpath_to_path_buf(root, &self.pkgpath)
    }

    /// Parse user input (CLI arg, `kcl.mod` entry, relative import). The
    /// parser already has `fix_import_path` in `crates/config/src/vfs.rs`;
    /// wrapping it here gives every caller the same error reporting.
    pub fn parse(import_path: &str, root: &str, file: &str) -> Option<Self> {
        let pkgpath = kcl_config::vfs::fix_import_path(root, file, import_path);
        if pkgpath.is_empty() { None } else { Some(Self::new(pkgpath)) }
    }
}

impl From<&str> for PkgPath { ... }   // infallible; wraps
impl From<String> for PkgPath { ... }
```

### Where it lives

`crates/utils/src/pkgpath.rs` already hosts `pkgpath_to_path_buf`,
`pkgpath_to_rel_path_buf`, `rm_external_pkg_name`, and
`external_pkgpath_to_rel_path_buf`. `PkgPath` is the typed wrapper that
composes them. `kcl-utils` is the lowest layer that both `kcl-config`
(for `fix_import_path`) and `kcl-parser` / `kcl-sema` / `kcl-loader`
already depend on, so adding the type here doesn't introduce a new
crate dependency.

### Where it gets used

The conversion is structural — call sites get the type, not just the
helper. Concretely, in this order of priority:

1. **`fix_import_path` return value** (`crates/config/src/vfs.rs`).
   Today it returns `String`. Change it to return `Option<PkgPath>`
   (returning `None` for the "exceeds parent" case the function already
   signals by `""`). One-line signature change; all current callers
   (`kcl-loader`, `kcl-parser`, `kcl-sema`, `crates/tools/src/bundle`)
   get the type and the `to_path` method, so they stop calling
   `replace('.', "/")` on their own.

2. **`Metadata.packages: HashMap<String, Package>`** in
   `crates/driver/src/toolchain.rs`. The map key is a package name; the
   value's `manifest_path` is a `PathBuf`. The dotted pkgpath form of
   the import — `import my_pkg.sub.dir` — is computed by callers via
   `external_pkgpath_to_rel_path_buf`. After this RFC the dotted side is
   a `PkgPath` and the conversion to a `PathBuf` is `pkgpath.to_path(&manifest_path)`.
   The cache (`crates/config/src/cache.rs::get_pkg_realpath_from_pkgpath`)
   benefits automatically.

3. **`CompileUnitOptions` and the loader result types.** They are
   `(Vec<String>, Option<LoadProgramOptions>, Option<Metadata>)` today.
   Leaving the outer shape alone but converting the file list and
   `package_maps` keys to `Vec<PkgPath>` / `HashMap<PkgPath, _>` is the
   incremental migration; it forces every caller to go through
   `to_path` to get a `PathBuf`, which is the whole point.

4. **Toolchain inputs.** `Toolchain::fetch_metadata(PathBuf)` and
   `Toolchain::update_dependencies(PathBuf)` already take a `PathBuf`
   for the manifest directory — that's a filesystem path, not a
   pkgpath, so it stays as `PathBuf`. The PkgPath work is orthogonal
   here.

### Out of scope but informed by this design

- **Toolchain inputs to package manifests.** Once `Metadata.packages`
   has `PkgPath` keys, `get_real_path_from_external(tool, pkg_name:
   &PkgPath, ...)` becomes type-safe. (Same for the LSP
   `external_pkg_real_path`.)
- **Loader ↔ evaluator handoff.** The evaluator currently takes
  pkgpath strings in its schema/symbol tables. Migrating those is a
  separate, larger change; the typed boundary this doc proposes is a
  prerequisite but not the whole thing.
- **A real `VFS` trait.** When (if) someone picks up zong-zhe's 2024
  design, the `PkgPath` type from this doc is what the trait's
  `exists(pkgpath: PkgPath) -> bool` signature should take. Building the
  trait on top of the newtype means the newtype is the only thing that
  changes later.

## Migration plan

Each step is independently shippable behind the existing test suite.

1. **Land `PkgPath` in `kcl-utils`.** Add the type and `to_path`. Add
   unit tests. No call-site changes. (Same shape as `pkgpath_to_path_buf`
   landing in #2178.)

2. **Convert `fix_import_path` return.** Update its callers in
   `kcl-loader`, `kcl-parser`, `kcl-sema`, `crates/tools/src/bundle`,
   and `crates/driver/src/toolchain`. Each is a one-line signature
   change; the `String` they get today is just wrapped in
   `PkgPath::new` for free.

3. **Convert `Metadata.packages` map keys.** (Optional but cheap.)
   Forces the remaining `replace('.', "/")` / `pkgpath_to_*_path_buf`
   call sites in `driver::get_real_path_from_external` and
   `LSP::external_pkg_real_path` to go through the newtype.

4. **Convert `CompileUnitOptions` file list and `package_maps`.** This
   is the load-bearing change — every parser/sema/runner entry point
   flows through these, and the conversion to `PathBuf` happens at the
   boundary with the filesystem.

After step 4, no internal site should be able to write
`pkgpath.replace('.', "/")` without first calling
`pkgpath.to_path(&root)` and getting a `PathBuf`. The compiler's type
system enforces the design.

## Open questions

1. **Should `PkgPath` reject empty strings at construction?** Today
   `String` is accepted everywhere; a non-empty invariant would catch
   some bugs. Cost: a `Result`-returning constructor or a panic. Lean
   toward `Option` for the `parse` constructor and infallible `new`
   for internal callers that already validated.

2. **Do we need a separate `RelPkgPath` (always starts with `.`) and
   `AbsPkgPath` (never does)?** The current `fix_import_path` returns
   either depending on the input. A split would catch misuses earlier
   but doubles the API surface. Defer until there's a concrete
   caller that needs to enforce the distinction.

3. **What about pkgpaths that map to a single file (`a.b.c` →
   `a/b/c.k`)?** This is the bundle / `import_target` case. The
   `to_path` method should probably take an `Option<&str>` extension
   (default `"k"`) or expose a `to_file_path(root, &k)` companion.
   The 2024 `PkgPath::new_with_extension` sketch covered this; we can
   add it in step 1 of the migration if bundle work is in flight.

4. **Does this conflict with the LSP salsa/VFS layer?** No. The
   LSP-layer VFS holds `SourceFile` instances keyed by URI; `PkgPath`
   lives below the parser boundary. They don't see each other. The
   maintainer's 2026 status note already separates these as item 7
   (LSP) and "compiler-side path unification" (this doc).

## Alternatives considered

- **Status quo + more helpers.** Keeps landing fixes as bugs appear.
  Cost: every new caller is a new chance to get the conversion wrong;
  test coverage is per-call-site, not structural.
- **Adopt zong-zhe's 2024 design wholesale.** Right answer long-term,
  wrong answer now: it requires a `SourceMap`-shaped refactor across
  `kcl-parser`, `kcl-loader`, and the LSP layer that the maintainer
  explicitly deferred in 2026.
- **A new `vfs` crate.** Same as adopting the full design, minus the
  `SourceFile` wrapper. Still requires per-crate API changes that
  block on the maintainer's architectural decision.
