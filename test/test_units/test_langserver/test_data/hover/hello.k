schema Server:
    name: str
    image: str
    replica: int = 1

myServer = Server {
    name: "demo"
    image: "demo image"
    replica: 10
}

n = myServer.name