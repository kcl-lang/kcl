# KEP-1867: Dashes in identifiers

Status: **Draft** · Tracking: kcl-lang/kcl#1867 · Author: Peefy / kcl-lang

## Motivation

KCL currently rejects dashes inside identifiers. The motivating use case is
Kubernetes manifests, where keys like `api-Version` are pervasive and the
existing escape hatch (`["api-Version"] = ...`) is noisy at the top level
of a config:

```kcl
schema Config:
    ["api-Version"]: str   # ugly
```

Adding dashes would let the same file read like the YAML it eventually
renders to:

```kcl
schema Config:
    api-Version: str       # cleaner
```

The discussion thread (issue #1867) converged on three distinct sub-cases,
each with its own complexity and risk profile. This KEP sketches the
ordering recommended there.

## Sub-cases

### Phase 1 — Config keys (top-level dict keys)

The current `[key]: value` form keeps working unchanged; a new bare-form
`key: value` becomes legal at top-level only when the key starts with a
letter or `_` and contains `[A-Za-z0-9_-]`.

**Why top-level only:** restricting the surface keeps the lexer change
narrow and avoids the operator/identifier ambiguity that comes up in
expression context (`a - b` is subtraction; `a-b` might be a single name).

**Lexer change:** `eat_ident` accepts `-` *after the first character*, but
only when the next character would also be a valid identifier
continuation. That last-clause check is what keeps `a-3` from becoming a
single token (the `3` would otherwise be a numeric literal suffix).
Actually, the simpler rule — accept `-` whenever the *current* identifier
has at least one character already — preserves the existing arithmetic
behavior because every existing `x - y` expression has whitespace around
the operator.

### Phase 2 — Schema attributes

Schema attributes (`api-Version: str`) require a separate grammar change
in the schema-body parser, not just the lexer. The decision points are:

- do we allow bare `api-Version: str` *only* when it contains a dash, or
  unconditionally for any attribute name?
- do we update `schema_attr_ty` in `sema/src/resolver/attr.rs` to look up
  attributes by both `name` and `name.replace('-', '_')` for migration?

This is best done as a follow-up after Phase 1 ships, since the lexer
change is a prerequisite.

### Phase 3 — Module / pkgpath with dashes

Module paths (`import my-pkg.sub-mod`) need a KEP of their own because:

- the current pkgpath grammar is dot-separated identifier segments;
- the loader's path → pkgpath mapping uses literal file paths, so
  `my-pkg/main.k` would have to canonicalize to `my_pkg`;
- Cross-references between identifiers and pkgpaths in the resolver
  (`mod_ty.kind`, `PLUGIN_MODULE_PREFIX`) would need new handling.

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| `a-b` parses as one ident instead of `a - b` | only at top-level; expression parser already expects `Minus` between operands |
| YAML round-trip changes (`api-Version` -> `api_version`) | Phase 2 includes a name-mapping layer in `sema` and the schema-attr lookup |
| Visual confusion in mixed `api-Version` / `api_version` schemas | rejected by Phase 2 lookup; tooling warning |

## Out of scope

- Numeric literal suffixes (already delimited by the existing `eat_lit_suffix`)
- Emojis and other non-ASCII continuation characters (existing rule stays)
- Decorator / annotation names

## Status

This KEP exists to capture the discussion. Phase 1 implementation is
non-trivial because the lexer change needs parser-position feedback to
stay safe in expressions, so a follow-up PR that introduces such a
signal (likely a `cursor.expect_config_key()` hint or equivalent) is
required before any code lands.