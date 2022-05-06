import pkg

age = 1

schema MainProtocol:
    var: int
    age: int

schema MainMixin for MainProtocol:
    var: int

rule Base:
    age > 0

rule Main[var, default=1](Base, pkg.PkgRule) for MainProtocol:
    age < 10

Main(1) {
    age = 1
}
