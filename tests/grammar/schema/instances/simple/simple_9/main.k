# Test case for multiple schema instances
# This test verifies that creating multiple instances
# of the same schema correctly counts all instances.

schema Config [name]:
    data: {str:str} = {
        key = "value"
    }
    metadata: {str:str} = {
        name = name
    }

# Create first instance
config1 = Config("config-1") {
    data.env = "production"
}

# Create second instance
config2 = Config("config-2") {
    data.env = "staging"
}

# Create third instance
config3 = Config("config-3") {
    data.env = "development"
}

# Should return 3 instances
instances_count = len(Config.instances())
instances_list = Config.instances()
