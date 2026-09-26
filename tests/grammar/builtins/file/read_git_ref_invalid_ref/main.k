import file

# A ref that does not exist in the repository is a hard error that
# reports the underlying `git show` failure.
a = file.read("asset.txt:NOT_A_REAL_REF")
