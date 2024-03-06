name = "volume"
configMapVolumes1: [{str:{str:}}] = [{
    name = {
        name = name
    } for _ in range(1)
}]

configMapVolumes2: {str:[{str:}]} = {name = [
    {
        name = name
    } for _ in range(1)
]}
