schema Service:
    port: {str: int|str}

srv_0 = Service {
    port: {
        name: "metrics",
        protocol: "TCP",
        target: 8080,
        containerPort: 8080
    }
}
