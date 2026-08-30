# `@info(type="attr")` mixed with regular attributes in the same
# schema should still let the schema round-trip through YAML without
# any side-channel marker (the marker is gated behind
# `emit_attribute_metadata` which is off for plain YAML output).
schema Button:
    @info(type="attr")
    name: str
    label: str
    @info(type="attr")
    onclick: str

ok = Button {
    name = "submit"
    label = "Submit"
    onclick = "handleSubmit()"
}