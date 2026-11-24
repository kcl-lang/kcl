#!/usr/bin/env bash

# Remove the embedded LLVM bitcode sections from a Rust static library.
#
# rustc embeds the bitcode of every upstream crate - including the
# precompiled standard library - in the .llvmbc/.llvmcmd sections of each
# member of a staticlib. The bitcode exists so that a further LTO pass can
# still happen; cargo has already run that pass when it produced the
# archive, so in a shipped artifact it is dead weight.
#
# It is not harmless dead weight, either. Go 1.26.11 rejects any archive
# referenced by a #cgo LDFLAGS whose members carry such a section:
#
#   disallowed LTO section ".llvmbc" in #cgo LDFLAGS: .../libkcl.a
#
# The shared libraries do not have the problem - the linker drops the
# non-allocated bitcode sections when it produces a .so - but an archive
# keeps every section of every member, so only the static library needs
# this.
#
# Usage: strip-llvm-bitcode.sh <archive.a> [<archive.a> ...]

set -euo pipefail

if [ "$#" -eq 0 ]; then
    echo "usage: $(basename "$0") <archive.a> [...]" >&2
    exit 2
fi

if ! command -v objcopy >/dev/null 2>&1; then
    echo "error: objcopy not found (install binutils)" >&2
    exit 1
fi

# The global symbols of an object file, as a sorted list of names.
#
# Only the global ones matter here: they are what the archive index
# advertises and what the linker resolves. The local ones are noise - every
# staticlib member repeats names like ltmp0, and the ARM64 mapping symbols
# ($d, $x, ...) show up or not depending on the host that reads them.
#
# readelf reads the symbol table of the ELF file itself. nm does not work
# for this: binutils loads an LLVM plugin to read the LTO bitcode of a
# section, the plugin on the runner is older than the rustc that wrote the
# bitcode, and the failure takes the symbol table of the whole file with
# it:
#
#   bfd plugin: LLVM gold plugin has failed to create LTO module: Opaque
#   pointers are only supported in -opaque-pointers mode (Producer:
#   'LLVM21.1.2-rust-1.91.1-stable' Reader: 'LLVM 14.0.0')
#   nm: <member>: no symbols
#
# That is exactly the bitcode this script removes, so the reading has to
# happen without the plugin.
symbol_names() {
    { readelf --wide --symbols "$1" 2>/dev/null || true; } |
        awk '$5 ~ /^(GLOBAL|WEAK|GNU_UNIQUE)$/ && $7 != "UND" && NF >= 8 {
                 name = $8
                 for (i = 9; i <= NF; i++) name = name " " $i
                 print name
             }' |
        LC_ALL=C sort -u
}

# The symbols an archive advertises to the linker, as a sorted list of
# names. This is the index of the archive itself, not the symbols of its
# members, so it is what a consumer of the library resolves against.
#
# nm prints the index as "<symbol> in <member>". The plugin error above
# lands on stdout as well and contains " in " too, so only a name without
# a space in it is taken as a symbol.
archive_symbols() {
    { nm --print-armap "$1" 2>/dev/null || true; } |
        awk 'index($0, " in ") > 0 {
                 sym = substr($0, 1, index($0, " in ") - 1)
                 if (sym != "" && sym !~ /[[:space:]]/) print sym
             }' |
        LC_ALL=C sort -u
}

# The members of an archive, as a sorted list of names. The symbol table
# of a BSD ar archive is a member of its own and is not a part of the
# library, so it is left out.
member_names() {
    ar t "$1" | sed '/^__\.SYMDEF/d' | LC_ALL=C sort
}

count_lines() {
    wc -l < "$1" | tr -d ' '
}

for arg in "$@"; do
    if [ ! -f "$arg" ]; then
        echo "error: no such archive: $arg" >&2
        exit 1
    fi

    archive="$(cd "$(dirname "$arg")" && pwd)/$(basename "$arg")"
    work="$(mktemp -d)"
    out="$(mktemp -d)"

    member_names "$archive" > "$out/members.before"
    archive_symbols "$archive" > "$out/symbols.archive.before"

    # GNU ar skips the archive's own symbol table when extracting, so only
    # the object members land in the work directory. The member names are
    # unique, so no member overwrites another.
    ( cd "$work" && ar x "$archive" )

    # A BSD ar keeps its symbol table in a member of its own, and ar x
    # extracts it as a file. It is the archive's index, not a part of the
    # library, so it does not go back in.
    rm -f "$work"/__.SYMDEF*

    if [ "$(find "$work" -type f | wc -l | tr -d ' ')" -eq 0 ]; then
        echo "error: no members extracted from $archive" >&2
        rm -rf "$work" "$out"
        exit 1
    fi

    # The symbols of every member, before the strip. The check below is
    # per member, so that a member that loses a symbol cannot hide behind
    # another member that has the same name.
    mkdir -p "$out/symbols.before"
    for obj in "$work"/*; do
        symbol_names "$obj" > "$out/symbols.before/$(basename "$obj").txt"
    done

    for obj in "$work"/*; do
        objcopy --remove-section=.llvmbc --remove-section=.llvmcmd "$obj"
    done

    # 's' writes the archive symbol table, which is what makes the archive
    # linkable; without it the consumers of this library get undefined
    # references instead of code.
    ( cd "$work" && ar crs "$out/$(basename "$archive")" * )
    mv "$out/$(basename "$archive")" "$archive"

    member_names "$archive" > "$out/members.after"
    archive_symbols "$archive" > "$out/symbols.archive.after"

    # The archive has to keep its shape: the same members, the same
    # symbols, and no bitcode left anywhere in it. A release that ships a
    # truncated archive is worse than one that ships a bitcode-carrying one.
    if ! diff -u "$out/members.before" "$out/members.after"; then
        echo "error: the member list changed while stripping $archive" >&2
        rm -rf "$work" "$out"
        exit 1
    fi

    # The symbols the archive exports are what a consumer resolves against,
    # so they have to be the same after the strip. An empty index means the
    # index could not be read at all, and an unread index would make every
    # other check pass without comparing anything.
    if [ ! -s "$out/symbols.archive.before" ] || [ ! -s "$out/symbols.archive.after" ]; then
        echo "error: could not read the symbol index of $archive" >&2
        rm -rf "$work" "$out"
        exit 1
    fi
    if ! diff -u "$out/symbols.archive.before" "$out/symbols.archive.after"; then
        echo "error: the exported symbols changed while stripping $archive" >&2
        rm -rf "$work" "$out"
        exit 1
    fi

    mkdir -p "$out/symbols.after"
    changed=""
    for obj in "$work"/*; do
        name="$(basename "$obj").txt"
        symbol_names "$obj" > "$out/symbols.after/$name"
        if ! diff -q "$out/symbols.before/$name" "$out/symbols.after/$name" >/dev/null; then
            changed="${changed} ${name}"
        fi
    done
    if [ -n "$changed" ]; then
        echo "error: the global symbols changed while stripping $archive" >&2
        for name in $changed; do
            echo "--- ${name%.txt} ---" >&2
            diff -u "$out/symbols.before/$name" "$out/symbols.after/$name" | head -20 >&2
        done
        rm -rf "$work" "$out"
        exit 1
    fi

    if grep -q -a -e '\.llvmbc' -e '\.llvmcmd' "$archive"; then
        echo "error: LLVM bitcode sections are still present in $archive" >&2
        rm -rf "$work" "$out"
        exit 1
    fi

    # An archive without a symbol table cannot be linked at all, so the
    # index has to be there even though nothing else changed. GNU nm
    # prints the header of the index it reads; a BSD ar keeps the index in
    # a member of its own. The output goes to a file first: piping into
    # grep -q ends the pipe early, and the SIGPIPE that follows would
    # fail the check under pipefail.
    ar t "$archive" > "$out/members.raw" 2>/dev/null || true
    nm --print-armap "$archive" > "$out/armap.txt" 2>/dev/null || true
    index_ok=0
    if grep -q "^Archive index:" "$out/armap.txt"; then
        index_ok=1
    fi
    if [ "$index_ok" -eq 0 ] && grep -q '^__\.SYMDEF' "$out/members.raw"; then
        index_ok=1
    fi
    if [ "$index_ok" -ne 1 ]; then
        echo "error: archive symbol table is missing from $archive" >&2
        rm -rf "$work" "$out"
        exit 1
    fi

    echo "stripped LLVM bitcode from $archive ($(count_lines "$out/members.after") members)"
    rm -rf "$work" "$out"
done
