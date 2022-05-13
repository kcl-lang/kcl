import .models

server = models.Server {
    "name": "productPage",
    "workloadType": "Deployment",
    "pod": {
        "volumes": [{"name": "tmp", "emptyDir": {}}],
        "activeDeadlineSeconds": 600
    }
}
