type Int = int
type FloatUnionInt = float | Int
type BoolUnionFloatUnionInt = bool | FloatUnionInt
type StringUnionBoolUnionFloatUnionInt = str | BoolUnionFloatUnionInt

a: Int = 1
b: FloatUnionInt = 2.0
c: BoolUnionFloatUnionInt = True
d: StringUnionBoolUnionFloatUnionInt = "xx"
