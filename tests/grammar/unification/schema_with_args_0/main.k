schema Server[r: int]:
    replica: int = r

server: Server(1) {}
server: Server(2) {}
