# KEP-2022: Reference and function-pointer types

Status: **Draft** · Tracking: kcl-lang/kcl#2022 · Author: Peefy / kcl-lang

## Motivation

Issue #2022 asks for syntax-level support for:

* **Reference types** (`&T`) — typed memory addresses.
* **Function pointer types** (`fn(Arg) -> Ret`) — first-class
  callable values.

The stated motivation is "to support C code generation" — i.e. so
that KCL configs can model C-style interfaces (register maps, FFI
signatures, ISR vectors) and emit the corresponding `.c` / `.h`
files.

The use case is real but the KCL community has, so far, treated
KCL as a *configuration* language, not an embedded-systems
description language. Adding reference / function-pointer types is a
step toward the latter.

## What already exists

KCL today has:

* `int`, `float`, `bool`, `str`, `list`, `dict`, `schema`, `union`,
  `any`, `None`-as-a-value, and a literal-type family.
* `lambda` *values* — `lambda x: int -> int { x + 1 }` — usable as
  values today.
* Decorators, mixins, and rule-based validation.

What's missing:

* A way to write a type that says "this is an `int` accessed via a
  reference" or "this is a function pointer with this exact
  signature". Today `lambda` is a value but its type is `any` /
  `Callable`.

## Why the change is hard

Adding `&T` and `fn(...) -> T` to KCL touches every layer:

1. **Lexer / parser.** Two new token sequences (`&T`, the arrow
   form for fn types). Conflict resolution needed: `&` today is
   used for `&` as a binary bitwise AND in some contexts? — actually
   KCL doesn't have `&`, so this is a clean addition.
2. **AST.** New `TypeRef` variants.
3. **Sema.** Reference types imply aliasing rules; function-pointer
   types imply subtyping for signatures (parameter contravariance,
   return covariance). Both are well-studied but not free.
4. **Runtime.** "Reference" today in KCL would have to mean either
   "a pointer to an external resource managed by the host" (which
   doesn't fit KCL's sandbox model) or "a value-level handle into
   the runtime heap" (which raises GC/lifetime questions KCL
   currently avoids).

## Options

### Option A — Type-only, no runtime support

Add `&T` and `fn(...) -> T` as type *expressions* that can appear in
schema attributes and function signatures, but the values are
opaque `any`-typed handles. Useful for documentation and for code
generators that consume the type AST.

Pros:
* Smallest blast radius (parser + type checker only).
* Lets a code-generation tool read the schema and emit matching
  `.h` files.

Cons:
* The values are not really `&T`; they're `any`. Users would have to
  cast at the boundary.
* Doesn't match what most people expect from a "reference type".

### Option B — Typed handles with FFI escape hatch

`&T` and `fn(...) -> T` map to opaque handles that the host (Go,
Python, Node, .NET) registers. The runtime treats them as
`any`-shaped values; the host can do its own dereferencing. This
mirrors the existing `kcl_plugin_init` pattern.

Pros:
* Real semantic content for the types.
* No new runtime primitives — the host owns the lifecycle.

Cons:
* Requires every SDK to implement the deref hooks.
* KCL itself can't manipulate the values (good, but limits use).

### Option C — Full C codegen toolchain

Build a `kcl-to-c` compiler that emits `.c` / `.h` from a KCL
schema, including reference/function-pointer declarations. This is
the user's stated goal but it's a separate project.

Pros:
* Solves the actual ask.

Cons:
* Years of work; not a small PR.
* Needs maintenance forever — every KCL language change needs a
  codegen change.

## Recommendation

Land **Option A** first as a no-runtime-cost addition: parse and
type-check `&T` and `fn(...) -> T`, then expose them through the
schema-introspection API (`kcl query schema`) so a code-generation
tool can read them. Defer Option B until there's a concrete user
asking for typed handles; defer Option C until there's a
maintainer willing to own the codegen.

For the user's stated motivation (C code generation), Option A is
genuinely useful: a downstream tool like `kcl-to-c` can read the
schema, see that attribute `handler: fn(int) -> int`, and emit
`void (*handler)(int);` in the matching `.h` — without KCL itself
ever needing to do anything with the pointer at runtime.

## Open questions for the maintainer

1. **Notation.** `&T` for reference is conventional, but KCL's
   attribute syntax is `name: T`. Is `name: &T` unambiguous? Or do
   we need a keyword form (`ref T`)?
2. **Function-pointer notation.** `fn(int) -> int` clashes with
   the existing `lambda` keyword in KCL (`lambda x: int -> int`).
   Do we need a new keyword (`fn`) or a context-sensitive form?
3. **Mutability.** `&T` vs `&mut T`? KCL values are immutable by
   default; this might be a non-question, but the codegen user
   probably wants it.

## Status

This KEP exists so the next person who hits this issue can read
the trade-offs and start from a concrete plan rather than asking
again. No code PR is open.