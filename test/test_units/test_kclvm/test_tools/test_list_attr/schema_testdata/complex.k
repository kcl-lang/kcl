@info(version="v1")
schema User:
    @info(message="name")
    name: str = "default_name"
    age: int = 1
    homeTown?: HomeTown

schema HomeTown:
    province: str
    city: Custom
  
schema Custom:
    left: int
    right: int

schema Color:
    color: "Yellow" | "Blue" | "Green"
    listAttr: [int]
    dictAttr: {str:}
