# Test int() function with base parameter
# Hexadecimal with 0x prefix
a = int("0xe4", base=16)
# Hexadecimal with 0X prefix (uppercase)
b = int("0XFF", base=16)
# Hexadecimal without prefix
c = int("e4", base=16)
# Octal with 0o prefix
d = int("0o755", base=8)
# Octal without prefix
e = int("755", base=8)
# Binary with 0b prefix
f = int("0b1010", base=2)
# Binary without prefix
g = int("1010", base=2)
# Decimal (base 10, default)
h = int("10")
# Decimal with explicit base 10
i = int("10", base=10)
# Negative hex with prefix
j = int("-0x10", base=16)
