import konfig.base.pkg.kusion_models.kube.frontend
import konfig.base.pkg.kusion_models.kube.frontend.container

## aa2-vendor/konfig/base/pkg/kusion_models/kube/frontend

main = container.Main {
    name = "sss"
}
aa = typeof(main, full_name=True)
cc = typeof(frontend.Server {
    image = "nginx"
    mainContainer = {
        name = "nginx"
    }
}.mainContainer, full_name=True)
