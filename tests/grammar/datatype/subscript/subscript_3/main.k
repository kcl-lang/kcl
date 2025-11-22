main = {"args": [1, 2, 3, 4, 5, 6]}
args = main["args"]
args_unpack = [
    *args[0:2], 
    "1", "1", 
    *args[2:len(args)]
]
