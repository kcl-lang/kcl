# KEP-1831: Calling Python from KCL

Status: **Draft** · Tracking: kcl-lang/kcl#1831 · Author: Peefy / kcl-lang

## Motivation

Today KCL can only reach Python by having Python be the host: a
Python process boots `kcl_lib`, registers a plugin via `kcl_plugin_init`,
then invokes KCL files that import the registered plugin. The
author of #1831 wants the inverse — KCL files should be able to invoke
arbitrary Python without any Python-side boilerplate, so that existing
PyPI packages can be used as configuration primitives.

The motivation is real (Numpy/Scipy/Pandas/requests are useful config
back-ends) but the current architecture is intentionally the opposite
shape.

## Why the host-callable plugin exists

`crates/runtime/src/stdlib/plugin.rs` exposes a one-way interface:

* `kcl_plugin_init(fn_ptr)` — the host registers a function pointer.
* Inside KCL, `import kcl_plugin.foo` resolves to `kcl_plugin_invoke(...)`
  which calls back into the host.

This was the *whole* of the Python integration as of #495 ("refactor:
remove kclvm python CI tests, CLI and plugin related codes"). The Python
host (`kclvm`), the Python CLI wrapper, and the Python CI tests were
all removed in that PR. What's left is the thin FFI surface, because
removing it would also break the Go/Python/Node/.NET SDKs that still
use it.

The reason given on #495 was that the Python host was a maintenance
burden (CI flakiness, version matrix, packaging) and that the SDKs
satisfied the "use Python alongside KCL" use case on their own.

## What "call Python from KCL" would actually require

Three viable shapes, each with significant costs.

### Option A — Embed CPython via PyO3

Add `pyo3` (or `cpython`) as a dependency, embed the interpreter in
the `libkcl` process, expose a new system function `python.call(...)`:

```kcl
import python

result = python.call("json.dumps", {"a": 1})
```

Pros:
* Single binary, no external process.
* Lowest call latency.

Cons:
* +30-50 MB to the WASM binary (a deal-breaker for the `wasm32-wasip1`
  target — `kclvm` already ships a WASM build and this would push it
  past the 4 MB cap many bundlers impose).
* Versioning pain: the embedded interpreter pins CPython ≤ 3.12 or so
  for PyO3 compatibility, while users may have 3.13+.
* KCL has no "Python interpreter" concept today; we'd be adding one
  for one feature.
* Build matrix expands: every release needs to be tested against
  multiple Python versions × multiple platforms × `pyo3` ABI bumps.

### Option B — Shell out to a sidecar

A new system function `python.call(...)` spawns `python3 -c "<module>"`,
exchanges JSON over stdin/stdout. Same shape as the file/git work
in #2107.

Pros:
* No new dependencies in `kcl-runtime`.
* Uses the user's installed Python (whatever version they want).
* Works identically on the WASM build via WASI's `process` preopens.

Cons:
* Cold-start cost per call (mitigated by keeping a long-lived child
  process behind a pipe).
* The KCL evaluator has to be careful about call-time blocking — the
  scheduler needs a place to yield. Today `kcl-vm` is synchronous;
  this is the same kind of change the existing `regex` / `net`
  modules had to make.
* Sidecar lifecycle: how does `kcl run` discover / launch / stop the
  Python child? Who owns the `pip install` step? (#1831 wants PyPI
  packages — do we also need a `python.pip("requests")` builtin?)

### Option C — Reintroduce a maintained Python CLI wrapper

Effectively undo #495: publish `kclvm-python` again, with proper
maintainer ownership.

Pros:
* Matches the existing Go/Node/.NET SDK pattern.
* No new runtime features.

Cons:
* The CI/version-matrix burden that #495 cited is still there.
* Doesn't actually answer the ask — the user still needs to launch
  Python, they just don't need to write glue code.

## Recommendation

Either **Option B** or **none**. Option A is the most ergonomic but
introduces a runtime dependency the project has explicitly avoided.
Option C doesn't address the request.

For Option B, the minimum viable design is:

```kcl
# System function. Module = dotted path; attr = function name.
import python

# args/kwargs serialize to JSON; result comes back as the return value.
parsed = python.call("json.loads", s)
data   = python.eval("pandas.read_csv", "huge.csv")  # shorthand for call
```

Open questions that need maintainer input before any code lands:

1. **Process model.** Long-lived sidecar vs. one-shot spawn?
2. **Pip.** Is installing PyPI packages a KCL concern? If yes, where
   does the lockfile live (`kcl.mod` extension? `kcl.lock`?)
3. **WASM.** WASI's `process` preopens aren't available in browsers.
   Does this mean Option B is `native`-only, or do we accept that
   `kclvm` can't call Python in the browser?
4. **Security.** Today KCL has no I/O surface that lets user-supplied
   data reach the OS. A `python.call` would be the first. Does it need
   to be opt-in (e.g., behind a `--allow-python` flag like
   `python -I`)?

## Status

No implementation. This KEP exists to capture the trade-offs so the
discussion has a concrete shape next time it resurfaces.