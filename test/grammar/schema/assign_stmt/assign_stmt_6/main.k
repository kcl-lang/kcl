schema MySchema:
    metadata: {str:} = {}

    metadata.environment = "dev"


output = MySchema {
    metadata.environment = "qa"
    metadata.name = "config"
}
