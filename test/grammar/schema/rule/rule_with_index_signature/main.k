protocol XProtocol:
    [...str]: str
    alice: str

rule XRule for XProtocol:
    alice == "Alice", "expected Alice, got ${alice}"
    bob == "Bob", "expected Bob, got ${bob}"

p = XProtocol {
    alice = "Alice"
    bob = "Bob"
}

x = XRule {
    alice = "Alice"
    bob = "Bob"
}
