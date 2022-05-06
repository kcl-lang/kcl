# disallowed name
I = 1

#schema name
schema person:
    name : str = "kcl"

# mixin
mixin personMixin:
    name : str = "kcl"

# schema argument, camelCase
schema Person[PaP]:
    # schema attr, camelCase
    Age : int = 1
    name : str = "1"

# variable, rgx:  ^[^\W\dA-Z][^\W_]+$
pers_on = Person {
    age = 1
}

# protocol
protocol personProtocol:
    name : str = "kcl"

# TODO: just for test coverage， remove when fix generic_walk for IfStmt.elif_body: List[List[Stmt]]
if Ture:
    _c = 1
elif True:
    _c = 2
else:
    _c = 3

# rule name
rule SomeRule:
    i > 1

rule someRule:
    i > 1
