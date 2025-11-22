schema Metadata:
    environment?: str = "qa"
    region?: str
    name?: str

schema MySchema1:
    metadata?: Metadata = {}

    metadata.environment = "dev"


output = MySchema1 {
    metadata.name = "config"
}
