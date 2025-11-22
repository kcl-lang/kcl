schema Metadata:
    environment: str
    region: str
    name: str

schema MySchema1:
    metadata: Metadata

schema MySchema2(MySchema1):
    metadata: Metadata {environment = "dev"}

schema MySchema3(MySchema2):
    metadata: Metadata {region = "us-east-1"}

output = MySchema3 {metadata.name = "hello"}

