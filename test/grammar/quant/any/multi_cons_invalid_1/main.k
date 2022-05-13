schema app_conf:
    ports: [int | str]

    check:
        any port in ports {
            int(port) == 80 or int(port) == 8080
        }, "invalid port"


main = app_conf {
    ports: [9090]
}