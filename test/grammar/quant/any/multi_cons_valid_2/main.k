schema app_conf:
    ports: [int | str]

    check:
        any port in ports {
            (int(port) == 80 or int(port) == 8080) or (6000 <= int(port) <= 7000)
        }, "invalid port"


main = app_conf {
    ports: [6666, 9999]
}