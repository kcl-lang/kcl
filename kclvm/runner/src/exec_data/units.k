import units

_data = {k: 1Ki}

type Unit = units.NumberMultiplier

x0: Unit = 1M
x1 = x0
x2 = int(x0)
x3 = float(x0)
x4 = x0 if x0 else 1
x5 = x0
x6 = [1Ki, 1M][0]
x7 = _data["k"]
x8: 1M = 1M

x0str = str(1M)
x1str = str(x0)
x2str = str(int(x0))
x3str = str(float(x0))
x4str = str(x0 if x0 else 1)
x5str = "{}".format(x0)
x6str = "${[1Ki, 1M][0]}"
x7str = "${_data: #json}"

schema Data:
    x0: Unit = 1Mi
    x1: any = 1Ki
    x2: 1M | 1Ki = x1
    x3: Unit = 1Ki

data = Data()
