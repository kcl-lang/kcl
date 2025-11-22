ports = [1, 2, 3]

schema Data:
    port: [int]

data = Data {
    port = [port ** 2 for port in ports]
    port = [1, 2, 3]
}
