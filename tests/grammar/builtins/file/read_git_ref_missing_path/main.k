import file

# A path that does not exist at the pinned revision is a hard error.
a = file.read("missing.txt:HEAD")
