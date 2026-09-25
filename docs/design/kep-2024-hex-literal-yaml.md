# KEP-2024: Hex literal in YAML output

Status: **Draft** · Tracking: kcl-lang/kcl#2024 · Author: Peefy / kcl-lang

## Motivation

KCL accepts hex integer literals at parse time (`0x01FF` parses to 511),
but every YAML serializer in the runtime emits them as decimal:

```kcl
schema Cmd:
    id: int

c = Cmd { id = 0x01FF }
```

```yaml
id: 511
```

The motivating use case is custom serial-port commands, where the hex
form is the readable one. The maintainer (Peefy) originally proposed a
plugin-based solution; an assignee tried it but did not land a PR. This
KEP explores why a runtime-only solution isn't enough and what a
sensible in-tree path looks like.

## Why a runtime fix is not free

The hex info is lost at parse time:

```
LiteralKind::Int { base, empty_int }
    | (lexer)
v
token::Integer { symbol, suffix }
    | (ty.rs, parser)
v
ast::IntLiteralType { value: i64, suffix }
    | (sema type)
v
TypeKind::IntLit(i64)
    | (runtime)
v
Value::int_value(i64)
```

By the time `to_yaml_string_with_options` runs, only `i64` is left. There
is no `radix` field to consult, and adding one everywhere above is
roughly a 7-file change across parser, AST, sema, and runtime.

## Options

### Option A — Plugin (Peefy's first suggestion)

External Go / Python process reads the YAML and re-formats the chosen
fields. Pros: zero changes to kcl-lang/kcl. Cons: requires users to
manage the host runtime, and the assigneed PR stalled.

### Option B — Schema-level `@format` decorator

```kcl
schema Cmd:
    @format(radix=16)
    id: int = 0x01FF
```

The decorator is consulted during YAML encoding to choose the radix.
Carries its state through `Value` as an attached `format_hint: Option<FormatSpec>`.
Minimal impact on the lexer/parser; localized to YAML encoder.

### Option C — Carry radix through (the proper fix)

Add `radix: u8` to `IntLiteralType`, plumb through the type system to a
new `Value::int_literal` variant or attached metadata, and use it in the
YAML encoder whenever the radix is non-default. Touches ~7 files.

## Recommendation

Land **Option B** first as a stop-gap (smallest blast radius, solves the
niche use case), then revisit Option C as a cleanup once the metadata
plumbing matures. Option A remains available for users who prefer it.

## Test plan

For Option B:

- `to_yaml_string_with_options` honors `@format(radix=16)` on int attrs
- `is_valid_radix(16)` rejects values outside `[2, 36]`
- `to_yaml_string` (no options) and JSON output remain unchanged

For Option C (if pursued):

- Existing 174 runtime tests + new round-trip tests for hex/oct/binary
- Parser tests for `0x...`, `0o...`, `0b...` literals preserving radix

## Status

This KEP exists to capture the discussion. Implementation requires a
decision between B and C from a maintainer; no code PR is open yet.