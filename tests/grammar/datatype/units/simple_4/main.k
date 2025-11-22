import units

_data = {k: 1Ki}

x0: units.NumberMultiplier = 1M
x1: units.NumberMultiplier = x0
x2: int = int(x0)
x3: float = float(x0)
x4 = x0 if x0 else 1
x5 = x0
x6 = [1Ki, 1M][0]
x7 = _data["k"]

x0str = str(1M)
x1str = str(x0)
x2str = str(int(x0))
x3str = str(float(x0))
x4str = str(x0 if x0 else 1)
x5str = "{}".format(x0)
x6str = "${[1Ki, 1M][0]}"
x7str = "${_data: #json}"
