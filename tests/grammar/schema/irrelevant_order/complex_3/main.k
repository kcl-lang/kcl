schema BaseServer:
    name: str = option("name")
    container: str

schema ClassicServer(BaseServer):
    container: str = name

schema AppServer(ClassicServer):
    globalApp: str = "app"

schema UserServer(AppServer):
    name = "user"

app = UserServer {}
