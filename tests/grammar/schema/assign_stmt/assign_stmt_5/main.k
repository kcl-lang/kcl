schema MySchema:
    metadata: {str:} = {}

    metadata.environment = "dev"


output = MySchema {
    metadata.name = "config"
}
