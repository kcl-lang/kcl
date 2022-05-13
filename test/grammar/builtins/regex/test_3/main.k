import regex

memmatch = regex.findall("4Gi", r"^(\d{0,63})(E|P|T|G|M|K|Ei|Pi|Ti|Gi|Mi|Ki)$")
num = memmatch[0][0]
unit = memmatch[0][1]
