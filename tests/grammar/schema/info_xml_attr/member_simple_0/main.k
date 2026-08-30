# Test `@info(type="attr")` is accepted by the parser/sema and
# surfaces as schema runtime metadata without breaking instantiation.
schema TextView:
    @info(type="attr")
    "android:id": str
    @info(type="attr")
    "android:text": str
    description: str

view = TextView {
    "android:id" = "@+id/userNameTextView"
    "android:text" = "用户名"
    description = "Username field"
}