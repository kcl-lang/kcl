schema Deployment:
    [str]: any

schema EnvMandatorySchedulingMixin:
    """EnvMandatorySchedulingMixin"""
    if data.envMandatory:
        _envMandatorySchedulingTolerations = [
            {
                "key": "app-env",
                "operator"  : "Equal",
                "value"     : "$env_type"
                "effect"    : "NoSchedule"
            }
        ]
        _envMandatorySchedulingMatchExpressions = [
            {
                "key": "app-env",
                "operator": "In",
                "values": "$env_type"
            }
        ]

schema OverQuotaMixin:
    """OverQuotaMixin"""
    if data.overQuota:
        _overQuotaTolerations = [
            {
                "key": "cluster.k8s/is-over-quota",
                "operator": "Equal",
                "value": "true",
                "effect": "NoSchedule"
            }
        ]
        _overQuotaMatchExpressions = [
            {
                "key": "cluster.k8s/is-over-quota",
                "operator": "In",
                "values": ["true"]
            }
        ]

schema AppLogicPoolMixin:
    """AppLogicPoolMixin"""
    if data.appLogicPool:
        _appLogicPoolTolerations = [
            {
                "key": "mandatory.k8s/app-logic-pool",
                "operator": "Equal",
                "value": data.appLogicPool,
                "effect": "NoSchedule"
            }
        ]
        _appLogicPoolMatchExpressions = [
            {
                "key": "mandatory.k8s/app-logic-pool",
                "operator": "In",
                "values": [data.appLogicPool]
            }
        ]

schema AppConfigurationRender[inputData]:
    mixin [
        EnvMandatorySchedulingMixin,
        OverQuotaMixin,
        AppLogicPoolMixin
    ]

    data: {str:} = inputData

    _envMandatorySchedulingTolerations?: [] = None
    _overQuotaTolerations?: [] = None
    _appLogicPoolTolerations?: [] = None

    _envMandatorySchedulingMatchExpressions?: [] = None
    _overQuotaMatchExpressions?: [] = None
    _appLogicPoolMatchExpressions?: [] = None

    deployment = Deployment {
        affinity.requiredDuringSchedulingIgnoredDuringExecution.nodeSelectorTerms: [
            {
                matchExpressions: [
                    *_envMandatorySchedulingMatchExpressions,
                    *_overQuotaMatchExpressions,
                    *_appLogicPoolMatchExpressions,
                ]
            }
        ]
        tolerations: [
            *_envMandatorySchedulingTolerations,
            *_overQuotaTolerations,
            *_appLogicPoolTolerations,
        ] 
    }

result = AppConfigurationRender(inputData={
    envMandatory = True
    overQuota = True
    appLogicPool = True
}) {}
