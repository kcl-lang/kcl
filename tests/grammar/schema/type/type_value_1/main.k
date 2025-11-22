schema ConfigA:
    id: int = 1
    idStr: str = "id: {}".format(id)

schema ConfigB:
    id: int = 2
    idStr: str = "idStr: {}".format(id)

configs: {str:} = {
    A = ConfigA
    B = ConfigB
}
configA = configs.A()
configB = configs.B()
