import duration

half = duration.sub("1h", "30m")
one_day = duration.sub("2d", "1d")
zero = duration.sub("1h", "1h")
with_seconds = duration.sub("1m", "30s")
full = duration.sub("1d2h3m4s", "1d1h")
