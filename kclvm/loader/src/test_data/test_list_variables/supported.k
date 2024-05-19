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