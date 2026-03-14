# Element type mismatch: string element passed to int item parameter
result = reduce(lambda acc: int, item: int -> int { acc + item }, ["not an int"], 0)
