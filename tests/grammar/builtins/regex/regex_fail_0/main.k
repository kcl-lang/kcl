import regex

compiled = [regex.compile("(") for _ in range(2)]
matched = [regex.match("a", p) for p in ["a", "a", "("]]
