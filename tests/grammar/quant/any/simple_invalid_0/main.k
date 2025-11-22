schema app_conf:
    ports: [int | str]

    check:
        any port in ports {
            int(port) == 8080
        }, "no server defined"


main = app_conf {
    ports: [80]
}