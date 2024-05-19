list = [ _x for _x in range(20) if _x % 2 == 0]
list1 = [i if i > 2 else i + 1 for i in [1,2,3]]


dict = {str(i): 2 * i for i in range(3)}

func = lambda x: int, y: int -> int { x + y }

schema IfSchema:
    trueValue?: int
    falseValue?: int

if_schema = IfSchema {
    if True :
        trueValue: 1
    else :
        falseValue: 2
}