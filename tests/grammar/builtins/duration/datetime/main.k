import duration

start = "2024-01-15T10:00:00Z"
end = duration.add_to_datetime(start, "1h30m")
overnight = duration.add_to_datetime(start, "2d")
before = duration.sub_from_datetime(start, "30m")
month_end = duration.add_to_datetime("2024-02-28T23:00:00Z", "2h")
fractional = duration.add_to_datetime(start, "1.5s")
