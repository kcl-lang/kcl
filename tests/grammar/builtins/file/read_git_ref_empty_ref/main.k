import file

# A trailing colon with an empty suffix is not treated as a git ref
# either: the string is read as a plain (non-existent) filesystem path.
a = file.read("asset.txt:")
