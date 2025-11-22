import datetime

assert datetime.validate("2024-08-26", "%Y-%m-%d") # Valid date
assert datetime.validate("1998-06-06", "%Y-%m-%d") # Valid date
assert datetime.validate("2023-12-31", "%Y-%m-%d") # Valid date
assert datetime.validate("2000-01-01", "%Y-%m-%d") # Valid date

assert not datetime.validate("2024-13-26", "%Y-%m-%d") # Invalid month
assert not datetime.validate("2024-06-32", "%Y-%m-%d") # Invalid day
assert not datetime.validate("2024-08-26abc", "%Y-%m-%d") # Additional characters
assert not datetime.validate("not-a-date", "%Y-%m-%d") # Non-date string
assert not datetime.validate("2024-08", "%Y-%m-%d") # Missing day
assert not datetime.validate("08-26", "%Y-%m-%d") # Missing year
assert not datetime.validate("2024-08-26 10:00", "%Y-%m-%d") # Extra components
assert datetime.validate("2024-01-01", "%Y-%m-%d") # Starting year of the new year
assert datetime.validate("2024-02-29", "%Y-%m-%d") # Valid leap year date
assert not datetime.validate("2023-02-29", "%Y-%m-%d") # Invalid leap year date

result: bool = datetime.validate("2000-01-01", "%Y-%m-%d") # Valid date
