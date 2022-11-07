import base.pkg.kusion_models.kube.frontend
import base.pkg.kusion_models.kube.frontend.container
import base.pkg.kusion_models.kube.frontend.serviceaccount as sa
import base.pkg.kusion_models.kube.frontend.volume

jobConfiguration: frontend.Job {
    # main container
    mainContainer = container.Main {
        name = "pi"
        command = ["perl",  "-Mbignum=bpi", "-wle", "print bpi(2000)"]
    }
    image = "perl"

    # configmap
    configMaps: [
        {
            name = "job-log-config"
            data = {
                log_level = "INFO"
            }
        }
    ]

    # volumes
    volumes: [
        {
            name = "config-vol"
            volumeSource = volume.ConfigMap {
                name = "job-log-config"
                items = [
                    {
                        key = "log_level"
                        path = "log_level"
                    }
                ]
            }
            mounts = [
                volume.Mount {
                    path = "/etc/config"
                }
            ]
        }
    ]

    # service account
    serviceAccount: sa.ServiceAccount {
        name = "job-example-sa"
    }
}