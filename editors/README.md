# KCL Editor Support

This directory hosts editor-side integrations that live alongside the
compiler (as opposed to the dedicated VS Code extension in
[`kcl-lang/vscode-kcl`](https://github.com/kcl-lang/vscode-kcl)).

## Sublime Text

[`sublime/KCL.sublime-syntax`](sublime/KCL.sublime-syntax) is a TextMate
grammar (YAML) for KCL. Drop the `sublime/` folder into your Sublime
Text `Packages/User` directory (or package it as a Package Control
package) and `.k` / `.kcl` files will get syntax highlighting, code
folding, and symbol-based navigation.

The grammar is also consumable by tools that parse TextMate grammars,
such as [`bat`](https://github.com/sharkdp/bat) and
[`delta`](https://github.com/dandavison/delta), so `git diff` of KCL
files renders with the same highlighting you see in Sublime.

### Scope map (summary)

| Scope                         | What it covers                          |
| ----------------------------- | --------------------------------------- |
| `keyword.control.kcl`         | Reserved keywords (`schema`, `lambda`, `if`, …) |
| `constant.language.kcl`       | `True`, `False`, `None`, `Undefined`    |
| `storage.type.kcl`            | Built-in types (`int`, `str`, `bool`, …) |
| `support.function.builtin.kcl`| Built-in functions (`len`, `print`, …)  |
| `constant.numeric.kcl`        | Integer / float literals (incl. SI suffixes) |
| `string.quoted.*.kcl`         | Quoted strings (single / double / raw / triple / doc) |
| `comment.line.number-sign.kcl`| `#` comments                            |
| `variable.other.kcl`          | Identifiers                             |
