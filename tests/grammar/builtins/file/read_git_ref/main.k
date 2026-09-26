import file

# A plain read does not involve git at all: it always returns the
# on-disk file, even when the file has uncommitted changes.
plain = file.read("asset.txt")

# ":HEAD" reads the latest committed revision of the file from the
# enclosing git repository.
head = file.read("asset.txt:HEAD")

# A tag pins the file to a specific revision, which may differ from
# both the on-disk file and HEAD.
pinned = file.read("asset.txt:v1.0")
