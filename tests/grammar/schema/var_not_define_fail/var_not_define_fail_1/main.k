schema Base:
    cc: int = 1

schema Person(Base):
    image: str

    check:
        regex.match(image, "^[a-zA-Z]+:\d+\.\d+\.\d+$"), "image name should be like 'nginx:1.14.2'"
