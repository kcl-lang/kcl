import .moduleB

schema SchemaA(moduleB.SchemaB):
    field_A: str = "A"
    field_A_int: int = 1
