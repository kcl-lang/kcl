import duration

doubled = duration.mul("30m", 2)
scaled = duration.mul(d="500ms", n=3)
half = duration.div("1h", 2)
half_sec = duration.div("1s", 2)
zero = duration.mul("1h", 0)
gt = duration.cmp("2h", "30m")
lt = duration.cmp("30m", "2h")
eq = duration.cmp("PT1H", "60m")
eq_ms = duration.cmp("500ms", "0.5s")
