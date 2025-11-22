@deprecated()
schema ObsoleteSchema:
    @deprecated(version="1.16", reason="use firstName and lastName instead", strict=True)
    attr: str

