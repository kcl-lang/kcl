input = {
    "k0": {
        "value": "v0"
    },
    "k1": {
        "value": "v1"
    }
}

output = [{
    "metadata": {
        "name": k
    },
    "spec": {
        "data": {
            "type": "mock_resource",
            "value": input[k]["value"]
        }
    }
} for k in input]
