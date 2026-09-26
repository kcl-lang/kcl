import file

# file.readbase64() accepts the same "path:ref" syntax as file.read():
# plain read from disk, latest committed revision via ":HEAD", and a
# pinned revision via a tag.
plain_b64 = file.readbase64("asset.txt")
head_b64 = file.readbase64("asset.txt:HEAD")
pinned_b64 = file.readbase64("asset.txt:v1.0")
