from pathlib import Path


def module_name(file_path: str) -> str:
    return Path(file_path).with_suffix("").name
