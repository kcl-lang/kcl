import file

# A suffix containing characters outside the ref grammar (here a
# space) is not treated as a git ref: the whole string falls back to
# a plain filesystem path, which does not exist.
a = file.read("asset.txt:bad ref")
