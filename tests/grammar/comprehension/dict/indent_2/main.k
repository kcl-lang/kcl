
dicts = { \
    str(k) \
    : \
    k if k in [1, 2] \
    else k * 100 \
    for k in range(6) \
}
