import datetime

# Issue #2057: `now` accepts an optional `ticks` (seconds since the Unix epoch)
# so that arbitrary past or future instants can be rendered, not just the
# current time. The `%s` directive round-trips the epoch independently of the
# local time zone, so these results are deterministic.
epoch1 = datetime.now("%s", 1000000000)
epoch2 = datetime.now("%s", 1700000000)
epoch3 = datetime.now("%s", ticks=1000000000)

# Rendering an explicit instant produces a valid, parseable date.
valid: bool = datetime.validate(datetime.now("%Y-%m-%d %H:%M:%S", 1000000000), "%Y-%m-%d %H:%M:%S")

# Without `ticks`, the existing behavior is unchanged (current time formatted).
current_ok: bool = datetime.validate(datetime.now("%Y-%m-%d"), "%Y-%m-%d")
