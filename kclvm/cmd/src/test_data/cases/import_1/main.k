import kcl4
import kcl4.container


main = container.Main {
    name = "sss"
}

appConfiguration1 = kcl4.Server {
    mainContainer = main
}

aa = typeof(main, full_name=True)
cc = typeof(appConfiguration1.mainContainer, full_name=True)

