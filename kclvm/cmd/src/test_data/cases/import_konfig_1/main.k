import konfig.base.pkg.kusion_models.kube.frontend
import konfig.base.pkg.kusion_models.kube.frontend.container

## aa2-vendor/konfig/base/pkg/kusion_models/kube/frontend

main = container.Main {
    name = "sss"
}
aa = typeof(main, full_name=True)
appConfiguration: frontend.Server {
    image = "nginx"
    mainContainer = {
        name = "nginx"
    }
}
cc = typeof(appConfiguration.mainContainer, full_name=True)
