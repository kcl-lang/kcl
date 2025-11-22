service = {
    "name": "name",
    "labels": {
        "key": "value"
    }
}

result = {
    "metadata": {
        "name": service["name"],
        "labels": {
            **(dict(service["labels"]) if "labels" in service else {}),
            **{"app": service["name"]}
        }
    }
}
