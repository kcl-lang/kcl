# Issue #1737: a config literal nested in a list-typed schema attribute default
# value must be type-checked against its element schema at compile time. Here
# `name_typo` is not an attribute of `Item`, so a compile error is expected.
schema Config:
    conditions?: [Item] = [{name_typo: "kcl"}]

schema Item:
    name: str
