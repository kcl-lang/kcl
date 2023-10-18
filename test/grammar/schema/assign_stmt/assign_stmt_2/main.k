schema Metadata:
    environment?: str = "qa"
    region?: str
    name?: str

schema MySchema1:
    metadata?: Metadata = {
        name = "config"
    }
    metadata.environment = "dev"
    metadata.region = "us-east-1"

output = MySchema1 {}
