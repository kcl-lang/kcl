schema Person:
    name: str
    relatives: [Family]

schema Family:
    relation: str
    name: str

schema Rule:
    kind: str = "Rule"
    access: [str]
    entity: Person
    __settings__: {str:} = {
        "output_type": "STANDALONE"
    }

schema RuleMixin:
    if rule:
        ruleModels = [{
            "entity": {
                "name": rule["name"],
                "relatives": rule["relatives"]
            },
            "access": rule["access"]
        }]

schema Model:
    mixin [RuleMixin]
    rule: {str:}
    ruleModels: [Rule]

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
