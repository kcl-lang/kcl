import duration

start = "2024-01-15 10:00:00.1705312800"
length = duration.add(duration.minute(10), duration.minute(30))
end = duration.add_to_datetime(start, length)
back = duration.sub_from_datetime(start, "30m")
span = duration.between("2024-01-15T10:00:00Z", "2024-01-17T12:30:00Z")
same = duration.between("2024-01-15T10:00:00Z", start)
frac = duration.between("2024-01-15T10:00:00Z", "2024-01-15T10:00:01.500Z")
subsec = duration.add_to_datetime("2024-01-15T10:00:00Z", "500ms")
