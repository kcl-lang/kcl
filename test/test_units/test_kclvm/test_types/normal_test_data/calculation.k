schema Data:
    [str]: str
    name: str

# 1. Binary Operator
# Add
add_a = 1 + 1
add_b = 1 + 1.1
add_c = 1.1 + 1.1
add_d = 1.1 + 1
add_e = True + 1
add_f = 1 + False
add_g = "s" + "s"
add_h = [0] + [1]
add_i = 1 + add_b
add_j = add_c + 1
add_k = add_e + add_f
add_l = 1 + 1 + 1
add_m = 1 + 1.1 + 1
add_n = 1.1 + True + 1 + 1.0
# Sub
sub_a = 1 - 1
sub_b = 1 - 1.1
sub_c = 1.1 - 1.1
sub_d = 1.1 - 1
sub_e = True - 1
sub_f = 1 - False
sub_g = 0 - 0.0
sub_h = True - False
sub_i = 1 - sub_b
sub_j = sub_c - 1
sub_k = sub_e - sub_f
sub_l = 1 - 1 - 1
sub_m = 1 - 1.1 - 1
sub_n = 1.1 - True - 1 - 1.0
# Mul
mul_a = 1 * 1
mul_b = 1 * 1.1
mul_c = 1.1 * 1.1
mul_d = 1.1 * 1
mul_e = True * 1
mul_f = 1 * False
mul_g = "s" * 2 + 2 * "s"
mul_h = 2 * [0] + [0] * 2
mul_i = 1 * mul_b
mul_j = mul_c * 1
mul_k = mul_e * mul_f
mul_l = 1 * 1 * 1
mul_m = 1 * 1.1 * 1
mul_n = 1.1 * True * 1 * 1.0
mul_o = (1 if mul_n else "1") * 2
# Div
div_a = 1 / 1
div_b = 1 / 1.1
div_c = 1.1 / 1.1
div_d = 1.1 / 1
div_e = False / 1
div_f = 1 / True
div_g = False / True / 1.0
div_h = 1.0 / 1 / True / True
div_i = 1 / div_b
div_j = div_c / 1
div_k = div_e / div_f
div_l = 1 / 1 / 1
div_m = 1 / 1.1 / 1
div_n = 1.1 / True / 1 / 1.0
# FloorDiv
floor_div_a = 1 // 1
floor_div_b = 1 // 1.1
floor_div_c = 1.1 // 1.1
floor_div_d = 1.1 // 1
floor_div_e = False // 1
floor_div_f = 1 // True
floor_div_g = False // True // 1.0
floor_div_h = 1.0 // 1 // True // True
floor_div_i = 1 // floor_div_b
floor_div_j = floor_div_c // 1
floor_div_k = floor_div_e // floor_div_f
floor_div_l = 1 // 1 // 1
floor_div_m = 1 // 1.1 // 1
floor_div_n = 1.1 // True // 1 // 1.0
# Mod
mod_a = 1 % 1
mod_b = 1 % 1.1
mod_c = 1.1 % 1.1
mod_d = 1.1 % 1
mod_e = False % 1
mod_f = 1 % True
mod_g = False % True % 1.0
mod_h = 1.0 % 1 % True % True
mod_i = 1 % mod_b
mod_j = mod_c % 1
mod_k = mod_e % mod_f
mod_l = 1 % 1 % 1
mod_m = 1 % 1.1 % 1
mod_n = 1.1 % True % 1 % 1.0
# Pow
pow_a = 1 ** 1
pow_b = 1 ** 1.1
pow_c = 1.1 ** 1.1
pow_d = 1.1 ** 1
pow_e = False ** 1
pow_f = 1 ** True
pow_g = False ** True ** 1.0
pow_h = 1.0 ** 1 ** True ** True
pow_i = 1 ** pow_b
pow_j = pow_c ** 1
pow_k = pow_e ** pow_f
pow_l = 1 ** 1 ** 1
pow_m = 1 ** 1.1 ** 1
pow_n = 1.1 ** True ** 1 ** 1.0
# LShift
lshift_a = 1 >> 0
lshift_b = 0 >> 1
lshift_c = 2 >> 2
lshift_d = 2 >> 1
lshift_e = 1 >> 2
# RShift
rshift_a = 1 << 0
rshift_b = 0 << 1
rshift_c = 2 << 2
rshift_d = 2 << 1
rshift_e = 1 << 2
# BitOr
bitor_a = 1 | 2
bitor_b = 2 | 3
bitor_c = 0xAb | 0xbA
bitor_d = [] | []
bitor_e = {} | {}
bitor_f = [1] | [2]
bitor_g = {key: "value1"} | {key: "value2"}
bitor_h = bitor_e | bitor_g
bitor_i = bitor_e | None
bitor_j = bitor_f | None
bitor_k = None | {key: "value2"}
bitor_l = Data {name: "v"} | {name: "v"}
bitor_m = Data {name: "v"} | Data {name: "v"}
# BitXOr
bitxor_a = 1 ^ 2
bitxor_b = 2 ^ 3
bitxor_c = 0xAb ^ 0xbA
bitxor_d = bitxor_a & 0xFF
bitxor_e = 0xFF & bitxor_b
bitxor_f = bitxor_c & bitxor_d
# BitAdd
bitand_a = 1 & 2
bitand_b = 2 & 3
bitand_c = 0xAb & 0xbA
bitand_d = bitand_a & 0xFF
bitand_e = 0xFF & bitand_b
bitand_f = bitand_c & bitand_d
# And
logicand_a = 1 and 2
logicand_b = "" and 2
logicand_c = None and 2.0 and "" and Undefined
logicand_d = logicand_a == 1 and 2.0 and ""
logicand_e = logicand_b and False and {} and []
# Or
logicor_a = 1 or 2
logicor_b = "" or 2
logicor_c = None or 2.0 or "" or Undefined
logicor_d = logicor_a == 1 or 2.0 or ""
logicor_e = logicor_b or False or {} and []
# 2. Compare Operator
# Eq
compare_eq_a = 1 == 1
compare_eq_b = 1 == 1.0
compare_eq_c = 1.0 == 1
compare_eq_d = True == False
compare_eq_e = None == False
compare_eq_f = None == Undefined
# NotEq
compare_not_eq_a = 1 == 1
compare_not_eq_b = 1 == 1.0
compare_not_eq_c = 1.0 == 1
compare_not_eq_d = True == False
compare_not_eq_e = None == False
compare_not_eq_f = None == Undefined
compare_not_eq_g = [] == []
compare_not_eq_h = {} == {}
compare_not_eq_i = "" == ""
compare_not_eq_j = compare_not_eq_h == True
# Lt
compare_lt_a = 1 < 2
compare_lt_b = 1 < 2.0
compare_lt_c = 1.0 < 2
compare_lt_d = 1.0 < 2.0
compare_lt_e = compare_lt_a < compare_lt_d
compare_lt_f = "s" < "ss"
compare_lt_g = [1] < [2]
# LtE
compare_lte_a = 1 <= 2
compare_lte_b = 1 <= 2.0
compare_lte_c = 1.0 <= 2
compare_lte_d = 1.0 <= 2.0
compare_lte_e = compare_lte_a <= compare_lte_d
compare_lte_f = "s" <= "ss"
compare_lte_g = [1] <= [2]
# Gt
compare_gt_a = 1 > 2
compare_gt_b = 1 > 2.0
compare_gt_c = 1.0 > 2
compare_gt_d = 1.0 > 2.0
compare_gt_e = compare_gt_a > compare_gt_d
compare_gt_f = "s" > "ss"
compare_gt_g = [1] > [2]
# GtE
compare_gte_a = 1 >= 2
compare_gte_b = 1 >= 2.0
compare_gte_c = 1.0 >= 2
compare_gte_d = 1.0 >= 2.0
compare_gte_e = compare_gte_a >= compare_gte_d
compare_gte_f = "s" >= "ss"
compare_gte_g = [1] >= [2]
# Is
compare_is_a = 1 is 2
compare_is_b = 1 is 2.0
compare_is_c = 1.0 is 2
compare_is_d = 1.0 is 2.0
compare_is_e = compare_is_a is compare_is_d
compare_is_f = "s" is "ss"
compare_is_g = [1] is [2]
compare_is_h = compare_is_g is True
compare_is_i = None is None
compare_is_j = True is False
compare_is_k = compare_is_f is Undefined
# In
compare_in_a = 1 in [2]
compare_in_b = "key" in {}
compare_in_c = "s" in "ss"
compare_in_d = compare_in_a in [True]
compare_in_e = compare_in_d in {}
# Not
compare_not_a = 1 not 2
compare_not_b = 1 not 2.0
compare_not_c = 1.0 not 2
compare_not_d = 1.0 not 2.0
compare_not_e = compare_not_a not compare_not_d
compare_not_f = "s" not "ss"
compare_not_g = [1] not [2]
compare_not_h = compare_not_g not True
compare_not_i = None not None
compare_not_j = True not False
compare_not_k = compare_not_f not Undefined
# IsNot
compare_is_not_a = 1 is not 2
compare_is_not_b = 1 is not 2.0
compare_is_not_c = 1.0 is not 2
compare_is_not_d = 1.0 is not 2.0
compare_is_not_e = compare_is_not_a is not compare_is_not_d
compare_is_not_f = "s" is not "ss"
compare_is_not_g = [1] is not [2]
compare_is_not_h = compare_is_not_g is not True
compare_is_not_i = None is not None
compare_is_not_j = True is not False
compare_is_not_k = compare_is_not_f is not Undefined
# NotIn
compare_not_in_a = 1 not in [2]
compare_not_in_b = "key" not in {}
compare_not_in_c = "s" not in "ss"
compare_not_in_d = compare_not_in_a not in [True]
compare_not_in_e = compare_not_in_d not in {}
# 3. Unary Operator
# UAdd
unary_add_a = +1
unary_add_b = +1.0
unary_add_c = +True
unart_add_d = None
unart_add_e = +unart_add_d
# USub
unary_sub_a = -1
unary_sub_b = -1.0
unary_sub_c = -True
unart_sub_d = None
unart_sub_e = -unart_sub_d
# Invert
unary_invert_a = ~1
unary_invert_b = ~True
unary_invert_c = ~False
unart_invert_d = None
unart_invert_e = ~unart_invert_d
# Not
unary_not_a = not 1
unary_not_b = not 1.0
unary_not_c = not ""
unary_not_d = not False
unary_not_e = not []
unary_not_f = not {}
unary_not_g = not None
unary_not_h = not Undefined
