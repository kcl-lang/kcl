import file

# A path that escapes the module root via `..` is rejected by the
# file.* scope check (kcl-lang/kcl#2189) before any git lookup happens.
a = file.read("../outside.txt:HEAD")
