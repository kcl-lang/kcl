schema Data:
    labels?: {str:str}

data0 = Data {
    labels.key = "value"
}
data1 = Data {
    if True:
        labels.key = "value"
}
data2 = Data {
    if False:
        labels.key = "value1"
    else:
        labels.key = "value2"
}
data3 = Data {
    if True:
        if True:
            labels.key = "value1"
}
data4 = Data {
    if False:
        if False:
            labels.key = "value1"
        else:
            labels.key = "value2"
    else:
        labels.key = "value3"
}
