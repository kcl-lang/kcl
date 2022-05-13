schema RuleMixin:
    if rule:
        ruleModel = Rule {
            "entity": {
                "name": rule["name"],
                "relatives": rule["relatives"]
            },
            "access": rule["access"]
        }

schema Family:
    relation: str
    name: str

schema Person:
    name: str
    relatives: [Family]

schema Rule:
    kind: str = "Rule"
    access: [str]
    entity: Person
    __settings__: {str:} = {
        "output_type": "STANDALONE"
    }

schema Model:
    mixin [RuleMixin]
    rule: {str:}
    ruleModel: Rule

alice = Model {
    "rule": {
        "name": "alice",
        "relatives": [
            {
                "relation": "father",
                "name": "Reese"
            }
        ],
        "access": ["read", "write"]
    }
}
