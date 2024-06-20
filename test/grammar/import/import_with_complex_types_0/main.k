import types

hosts0: {str:types.Host} = {
    foo: {host: "foo.example.net"}
    bar: {host: "bar.example.net"}
}

hosts1: {str:types.HostPort} = {
    foo: {host: "foo.example.net", port: 80}
    bar: {host: "bar.example.net", port: 80}
}
