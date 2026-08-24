# Regression test for kcl-lang/kcl#2130
#
# Bug: a lambda argument name (e.g. `config`) was leaking from the lambda
# call's local-variable set into the surrounding scope, so subsequent
# reads of `config.X` inside the same dict literal that contains the
# `LambdaCall(_) | {...}` sibling key resolved `config` itself to
# `Undefined` and crashed with:
#
#     invalid value 'UndefinedType' to load attribute 'X'
#
# This is exercised by the schema attribute pattern below: the dict
# comprehension has a `metadata = MetadataBuilder(config) | {...}` key
# and a sibling `spec = {...}` key whose value reads `config.workloadType`
# after an optional-chain that triggered the `or` short-circuit.

schema Server:
    name: str = "x"
    workloadType: str = "Deployment"
    services: [{str:}] = [{selector = None}]
    labels?: {str:str} = None
    selector?: {str:str} = None

MetadataBuilder = lambda config -> {str:} {
    {
        name: config?.name or "default"
        labels: config?.labels
    }
}

schema Outer:
    config: Server = Server {}
    kubernetes: {str:} = {Deployment: [{spec: {selector: {matchLabels: {app: "test"}}}}]}

    workloadAttributes: [{str:}] = [
        {
            metadata = MetadataBuilder(config) | {name = "override"}
            spec = {
                selector = _s?.selector or kubernetes?[config.workloadType]?[0]?.spec?.selector?.matchLabels
            }
        } for _i, _s in config.services
    ]

o = Outer {}