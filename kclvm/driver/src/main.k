[item | {
      if item.kind == "Job":
          spec.template.metadata.labels = {
              meta.k8s.alipay.com/zone = tydd
              sigma.ali/app-name = sigmaetcd
              sigma.ali/deploy-unit = sigmaetcd-prod
              sigma.ali/instance-group = sigma3-alipay-master
              sigma.ali/site = tydd
          }
    } for item in option("items")]