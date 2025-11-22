assert True if False
assert 1 == 1 if False
_x = "good case"
assert _x == "good case" if not _x, "_x need to be 'good case'"
