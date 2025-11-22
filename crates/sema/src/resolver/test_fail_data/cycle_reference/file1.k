import .file2

schema SchemaBase(SchemaSub):

schema SchemaSub(SchemaBase):

rule RuleBase(RuleSub):
    True

rule RuleSub(RuleBase):
    True

schema A(B):
    name: str

schema B(C):
    name: str

schema C(A):
    name: str
