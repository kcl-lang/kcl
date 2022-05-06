schema Data:
    field: Field

schema Field:
    id?: int 

schema Config:
    data0: Data = {
        field: {
            id: 1
        }
    }
    data1: Data = {
        field: []
    }

x0 = Config {}
