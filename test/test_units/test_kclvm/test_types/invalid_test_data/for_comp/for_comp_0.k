data1 = {"k1": "v1", "k2": "v2"} 
data2 = {"k3": "v3", "k4": "v4"}
# Error: dict unpacking cannot be used in dict comprehension
dataNew = {**{"${k1}": v1, "${k1}": v2, "${k2}": v1, "${k2}": v2} for k1, v1 in data1 for k2, v2 in data2}
