age = 1

@deprecated
rule DeprecatedRule:
    age == 0

schema RuleProtocol:
    age: int

rule Base:
    age > 0

rule Main(Base) for RuleProtocol:

Main {}
