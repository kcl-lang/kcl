config: {str:} = {
    "map" = [
        {
            key = "value"
        }
    ]
    "values" = [
        "foo"
        "bar"
    ]
}
$map: [{str:str}] = config.map
