schema Data:
    name: str
    version?: str

data1 = Data {
    name = data2.name
}

data2 = Data {
    name = "1"
    version = version
}

# Global version
version = "v0.1.0"
