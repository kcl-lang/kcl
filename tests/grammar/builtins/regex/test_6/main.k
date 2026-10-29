import regex

# The same patterns are used repeatedly and interleaved, so later calls
# reuse patterns that earlier calls compiled.
_digits = r"\d+"
_words = r"[a-z]+"
replaced = [regex.replace(s, p, "-") for s in ["a1b22", "c333d"] for p in [_digits, _words, _digits]]
split = [regex.split(s, _digits) for s in ["a1b22c", "x9y"]]
found = [regex.findall(s, _words) for s in ["a1b22c", "x9y"]]
matched = [regex.match(s, _digits) for s in ["123", "abc", "4"]]
searched = [regex.search(s, _words) for s in ["123", "abc"]]
compiled = [regex.compile(p) for p in [_digits, "(", _digits, "("]]
inline_flags = [regex.match("A", p) for p in ["a", "(?i)a", "a", "(?i)a"]]
replaced_limited = [regex.replace(string="a1b22", pattern=_digits, replace="-", count=n) for n in [0, 1, 2]]
split_limited = [regex.split(string="a1b22c", pattern=_digits, maxsplit=n) for n in [0, 1, 2]]
backreferences = [regex.findall("aa bb cc", r"(\w)\1") for _ in range(2)]
lookbehind = [regex.search(string=s, pattern=r"(?<=a)b") for s in ["ab", "cb", "ab"]]
unicode_matches = [regex.findall(s, r"\p{Greek}+") for s in ["αβ1γ", "δ"]]
empty_matches = [regex.findall("ab", ""), regex.findall("", "a")]
