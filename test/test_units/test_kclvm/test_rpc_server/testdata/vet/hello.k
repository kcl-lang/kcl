
schema Person:
    key: str

    check:
        "value" in key  # 'key' is required and 'key' must contain "value"
