import file as libfile

schema FilesSchema:
    file: str

    check:
        libfile.exists(file) if file, "file not found ${file}"

v = FilesSchema {file = ""}
