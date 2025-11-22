schema Metadata:
    environment?: str = "qa"
    region?: str
    name?: str

schema MySchema1:
    metadata?: Metadata = {}

    metadata.environment = "dev"
    metadata.region = "us-east-1"

output = MySchema1 {
    metadata.name = "config"
}
