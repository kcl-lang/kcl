# KEP-1604: KCL ↔ Nixlang

Status: **Draft** · Tracking: kcl-lang/kcl#1604 · Author: Peefy / kcl-lang

## Motivation

Issue #1604 asks for either a KCL plugin for Nixlang or a KCL → Nix
transpiler, so users can write packages in KCL that consume
[nixpkgs](https://github.com/NixOS/nixpkgs) and produce Nix
configurations.

Two distinct use cases live under the same title:

1. **KCL → Nix transpiler**: KCL files compile to `.nix` expressions
   that `nix` can evaluate.
2. **KCL with a Nix library**: KCL files can `import nixpkgs` (via a
   plugin that talks to a Nix daemon / `nix eval`).

These are different projects with different cost profiles. Both are
real use cases — KCL is currently CNCF Sandbox and ships for
Kubernetes/IaC use cases where Nix already has users.

## Why neither is free

### Transpiler

KCL and Nix have very different semantics:

| | KCL | Nix |
|---|---|---|
| Evaluation | strict | lazy |
| Types | static, with schema | dynamic |
| Side effects | none in the language | system access via `builtins` |
| Modules | schemas + imports | attribute paths + `let` bindings |

The static-typing → dynamic-typing mapping loses information. A KCL
`schema` becomes a Nix `attrset` with no enforced shape; a KCL
constraint rule (`check: ...`) becomes either a Nix assertion (when
the user opts in to throwing) or a comment.

Concretely, the easy half is expressions: `a = b + c` maps to
`a = b + c;`. The hard half is everything around the expression —
schema declarations, rule checks, mixins, decorators.

### Nix library (plugin)

The Nix evaluation model needs a running Nix daemon (`nix-daemon` /
`nix eval --impure`). The same questions that block #1831 apply:

* KCL has no concept of "the host process owns a long-lived daemon".
* The `kclvm` WASM build can't talk to a Nix daemon in a browser.
* `nixpkgs` is large (~1 GB on disk after evaluation); caching and
  purity become KCL's problem.

## What *is* free: a small useful subset

If "KCL ↔ Nixlang" is read narrowly as **"I want to consume a single
Nix expression's output as a KCL value"**, the work is bounded:

```kcl
import nix

# `nix.eval` shells out to `nix eval --json '<expr>'` and decodes the
# result. Same shape as the file/git work in #2107.
nixos_version = nix.eval("(import <nixpkgs> {}).nixosVersions.stable")
```

This covers the case where a KCL config wants to *consume* a value
from Nix (e.g., "the latest stable nixpkgs commit") without trying
to compile KCL down to Nix. The plugin is a thin wrapper around
`nix eval --json`.

## Options

### Option A — `nix.eval` system function (small)

Adds a `nix` system module with `nix.eval(expr: str) -> any` that
shells out to `nix eval --json <expr>`. Mirrors the shape of
`file.read("path:ref")` from #2106. Doesn't ship a Nix daemon,
doesn't ship `nixpkgs`; the user is responsible for having `nix` on
`$PATH`.

### Option B — KCL → Nix transpiler

Write a new crate `kcl-nix` that emits `.nix` source. Substantial
work because:

* Schema → attrset mapping needs a decision (shape? optional vs.
  required?)
* Decorators (`@deprecated`) → `lib.warn` or `throw`?
* Rule checks → assertions?
* Mixins → `let` bindings?

This is the bulk of the issue but it's also the part most likely to
disappoint users: a "KCL in Nix" that drops the type system is not
KCL anymore.

### Option C — Nix-side plugin (`kcl` for Nixlang)

Write a Nix library that calls back into `libkcl` via FFI / JSON-RPC,
similar to the existing `terraform-nix` / `nix-eval` patterns.
Effectively Option A in the other direction.

## Recommendation

Ship **Option A** as the first step. It's small (~150 lines: `nix`
module + 2 FFI entries + tests), it matches the user's stated
motivating example, and it doesn't lock us into a Nix-side runtime.

Revisit Option B / C as separate KEPs once there are real users
asking for them.

## Status

No implementation. This KEP exists so the discussion has a concrete
shape next time it resurfaces.