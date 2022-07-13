age = 1

@deprecated
rule DeprecatedRule:
    age == 0

schema RuleProtocol:
    age: int

rule Main for RuleProtocol:
    True

Main {}
