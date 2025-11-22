schema app_conf:
    ports: [int | str]

    check:
        all port in ports {
            (int(port) == 80 or int(port) == 8080) or (6000 <= int(port) <= 7000)
        }, "invalid port"


main = app_conf {
    ports: [80, 8080, 6666]
}