protocol KubeResourceProtocol:
    svc: Service
    

schema Service:
    name: str

rule ServiceCheckRule for KubeResourceProtocol:
    svc.name != "123"
    

svc = Service {
    name: "123"
}

ServiceCheckRule {
    svc: svc
}