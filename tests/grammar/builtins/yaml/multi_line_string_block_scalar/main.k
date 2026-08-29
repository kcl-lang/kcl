# Issue kcl-lang/kcl#1894: the YAML representation of a multi-line triple-string
# (`"""\...\"\"`) must round-trip exactly, including its trailing newline, even
# when the string is the last value of the last field of the last mapping of
# the document. Previously, `value.plan()` unconditionally stripped the
# trailing `\n` that `serde_yaml::to_string` emits, which happened to be the
# block scalar's terminator, so the round-tripped value came back without
# its final newline.
schema Foo:
    multi_line_string_1: str
    multi_line_string_2: str

multi_line_string = """\
line1
line2
line3
"""

foo = Foo {
    multi_line_string_1: multi_line_string
    multi_line_string_2: """\
line1
line2
line3
"""
}