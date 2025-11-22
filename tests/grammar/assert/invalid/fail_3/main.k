_x = "good case"
assert _x == "bad case" if not _x, "x should be 'good case'"
assert _x == "bad case" if _x, "x should be 'good case'"
