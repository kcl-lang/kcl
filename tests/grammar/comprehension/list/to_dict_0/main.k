input = {
    "k0": "v0",
    "k1": "v1"
}

output = [{
    "metadata": {
        "name": k
    },
    "data": input[k]
} for k in input]
