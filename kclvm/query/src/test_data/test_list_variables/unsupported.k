list = [ _x for _x in range(20) if _x % 2 == 0]
list1 = [i if i > 2 else i + 1 for i in [1,2,3]]


dict = {str(i): 2 * i for i in range(3)}

func = lambda x: int, y: int -> int { x + y }

schema IfSchemaInner:
    innerValue: int

schema IfSchema:
    trueValue?: int
    falseValue?: int
    name: str
    age: int
    inner: IfSchemaInner
    inner2: IfSchemaInner

if_schema = IfSchema {
    if True :
        trueValue: 1
    else :
        falseValue: 2

    name: "name"
    age: 1
    inner: IfSchemaInner {
        innerValue: 1
    }

    inner2: {
        innerValue: 2
    }
}

if_schema1 = {
    if True :
        trueValue: 1
    else :
        falseValue: 2

    name: "name"
    age: 1
    inner: IfSchemaInner {
        innerValue: 1
    }

    inner2: {
        innerValue: 2
    }
}