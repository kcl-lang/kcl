import regex

split_unlimited = regex.split("a,b,c,d", ",")
split_zero = regex.split("a,b,c,d", ",", 0)
split_one = regex.split("a,b,c,d", ",", 1)
split_two = regex.split("a,b,c,d", ",", 2)
split_exact = regex.split("a,b,c,d", ",", 3)
split_more = regex.split("a,b,c,d", ",", 10)
