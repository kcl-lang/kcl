schema app_conf:
    ports: [int | str]

    check:
        any port in ports {
            int(port) >= 1024 and int(port) <= 65535
        }, "invalid port"


main = app_conf {
    ports: [-1]
}