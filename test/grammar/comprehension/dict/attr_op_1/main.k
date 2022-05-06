schema Data:
    x0: {str:} = {str(v) = {str(k) = v} for k, v in [1, 1, 1]}
    x1: {str:}  = {str(v) += [{str(k) = v}] for k, v in [1, 1, 1]}
    x2: {str:}  = {str(v): {str(k) = v} for k, v in [1, 1, 1]}

data = Data {}
