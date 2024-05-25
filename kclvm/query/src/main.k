a = [1, 2, 3, 4]
b = {
    "c": "d",
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
