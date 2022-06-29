schema Main:
    name?: str
    env?: [{str:}]

schema Probe:
    initialDelaySeconds?: int
    timeoutSeconds?: int
    periodSeconds?: int = 10
    successThreshold?: int
    failureThreshold?: int

schema AppConfiguration:
    appName: str
    image: str
    overQuota: bool = False
    resource: {str:}
    mainContainer?: Main
    labels: {str:}
    probe?: Probe

appConfiguration = AppConfiguration {
    appName: "kclvm"
    image: "kclvm/kclvm:v0.1.0"
    resource: {
        cpu: "4"
        disk: "50Gi"
        memory: "12Gi"
    }
    labels: {
        key: {
            key: "value"
        }
    }
    mainContainer: Main {
        name: "kclvm"
    }
    overQuota = True
    overQuota = True
    probe: Probe {}
}

appConfigurationUnification: AppConfiguration {
    appName: "kclvm"
    image: "kclvm/kclvm:v0.1.0"
    resource: {
        cpu: "4"
        disk: "50Gi"
        memory: "12Gi"
    }
    labels: {
        key: {
            key: "value"
        }
    }
    mainContainer: Main {
        name: "kclvm"
    }
    overQuota: True
}
