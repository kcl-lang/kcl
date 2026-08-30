# `@info(type=...)` with a role other than `attr` must be silently
# ignored so existing free-form uses of `@info` keep working.
schema Legacy:
    @info(type="documentation")
    name: str
    @info(type="note", author="alice")
    version: str

item = Legacy {
    name = "alpha"
    version = "1.0"
}