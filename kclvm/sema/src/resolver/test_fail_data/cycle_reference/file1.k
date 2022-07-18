import .file2

schema SchemaBase(SchemaSub):

schema SchemaSub(SchemaBase):

rule RuleBase(RuleSub):
    True

rule RuleSub(RuleBase):
    True
