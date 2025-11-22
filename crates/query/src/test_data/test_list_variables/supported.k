a = 1
a1 = 2
a3 = 3m

b1 = True
b2 = False

s1 = "Hello"

array1 = [1, 2, 3]

dict1 = {"a": 1, "b": 2}
dict2 = {
    "a": 1
    "b": {
        "c": 2
        "d": 3
    }
}

schema A:
    name: str
    ids: [int]
    data?: {str: {str: {str: int}}}

sha = A {
    name: "Hello"
    ids: [1, 2, 3]
    data: {
        "a": {
            "b": {
                "c": 2
            }
        }
    }
}

schema B:
    a: A

shb = B {
    a: {
        name: "HelloB"
        ids: [4, 5, 6]
        data: {
            "d": {
                "e": {
                    "f": 3
                }
            }
        }
    }
}

schema UnificationConf:
    name: str

uconfa = UnificationConf{
    name = "a"
}

uconfa : UnificationConf {
    name = "b"
}

schema C:
    a: A

c = C {
    a: {name: "Hello"}
    a: {ids: [7, 8, 9]}
}

schema Job:
    name: str

job = Job {
    name = "{}-{}".format("app", "test").lower()
}

select = a.b.c {
    a: 1
}

_part1 = {
    a = "b"
}

_part2 = {
    c = "d"
}

_list0 = [1, 2, 3]
_list1 = [4, 5, 6]
union_list = [*_list0, *_list1] 

a_dict = {**_part1, **_part2}  # {"a: "b", "c": "d"}