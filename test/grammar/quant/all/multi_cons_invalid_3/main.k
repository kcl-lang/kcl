schema app_conf:
    ports: [int | str]

    check:
        all port in ports {
            (int(port) >= 3000 and int(port) <= 4000) or (int(port) >= 6000 and int(port) <= 7000)
        }, "invalid port"


main = app_conf {
    ports: [3333, 6666, 2333]
}