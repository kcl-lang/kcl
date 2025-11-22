schema Host:
    host: str

schema HostPort(Host):
    port: int

hosts: {str:HostPort} = {
    foo: {host: "foo.example.net", port: 80}
}