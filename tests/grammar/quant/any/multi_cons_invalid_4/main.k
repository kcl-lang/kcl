schema app_conf:
    ports: [int | str]

    check:
        any port in ports {
            (int(port) >= 3000 and int(port) <= 4000) and (int(port) >= 6000 and int(port) <= 7000)
        }, "invalid port"


main = app_conf {
    ports: [2333]
}