schema DataA:
    id?: int = 1
    value?: str = "value"

schema DataB:
    name?: str = "DataB"

_dataFactory: {str:} = {
    A = DataA
    B = DataB
}
TypeA = _dataFactory["A"]
TypeB = _dataFactory["B"]
data0 = TypeA()
data1 = TypeB()
data2 = TypeA() {}
data3 = TypeB {}
data4 = TypeA {}
data5 = TypeB {}
