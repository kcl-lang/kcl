import duration

half_hour_plus_min = duration.add("30m", "60s")
overflow = duration.add("23h", "2h")
with_zero = duration.add("0s", "1h30m")
days = duration.add("1d", "1d")
full = duration.add("1d2h3m4s", "1d2h3m4s")
