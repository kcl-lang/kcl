
schema Label:
    service: str
    app: str

schema Service:
    services: [Label]

services = [{
    "name": "Alice"
}]

my_service = Service {
    services: [
        Label {
            service: service["name"],
            "app": "123"
        } for service in services
    ]
}
