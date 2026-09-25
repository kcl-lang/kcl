import duration

valid_combined = duration.is_valid("1h30m")
valid_day = duration.is_valid("2d")
invalid_unit = duration.is_valid("10x")
missing_unit = duration.is_valid("10")
out_of_order = duration.is_valid("30m1h")
not_a_number = duration.is_valid("abc")
