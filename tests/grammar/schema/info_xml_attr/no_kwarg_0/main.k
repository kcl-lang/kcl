# `@info()` with no kwargs is a no-op. The decorator must be
# accepted by the runtime but contribute no metadata, so existing
# usages without kwargs keep producing the same output.
schema Empty:
    @info()
    first: str
    @info
    second: str

out = Empty {
    first = "a"
    second = "b"
}