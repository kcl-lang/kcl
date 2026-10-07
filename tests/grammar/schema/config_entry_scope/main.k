schema Item:
    p: int
    q: int

schema Nested:
    a: int
    b: int

schema Mid:
    a: int
    nested: any
    c: int

# A config entry must shadow an attribute of the same name in the enclosing
# schema for the entries that follow it. See issue #1769.
schema Outer:
    p: int
    item: any = Item {
        p: p + 1
        q: p + 1
    }
    # The shadowing must not outlive the config expression.
    after: int = p

# The shadowing must survive the evaluation of a nested schema body and must be
# restored to the enclosing config entry when that nested config is left.
schema Top:
    a: int
    mid: any = Mid {
        a: a + 1
        nested: Nested { a: 50, b: a }
        c: a
    }

outer = Outer { p: 100 }
top = Top { a: 10 }

# A schema config entry rebinds the attribute it assigns, so the entries that
# follow it resolve to the rebound value, not to the attribute of the enclosing
# schema. The type checker must agree with the runtime here: `y` sees the
# rebound `x` and so must type-check against it.
schema Rebound:
    x: str
    y: str

schema RebindOuter:
    x: [int]
    item: any = Rebound {
        x: "hello"
        y: x
    }

# A key of a plain config expression is a dict field: it binds no name, so it
# must not shadow a schema attribute of the same name. Both `before` and
# `after` must resolve `config` to the schema attribute, whatever order the
# entries are written in. See issue #2212.
schema DictCfg:
    super_users: [str] = ["alice", "bob"]

schema DictOuter:
    config: DictCfg
    before: {str:any} = {
        authorization = {superUsers = config.super_users}
        config = {"num.threads" = "8"}
    }
    after: {str:any} = {
        config = {"num.threads" = "8"}
        authorization = {superUsers = config.super_users}
    }

rebind_outer = RebindOuter { x: [1, 2] }
dict_outer = DictOuter { config = DictCfg {} }
