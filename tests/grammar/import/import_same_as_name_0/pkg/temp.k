import pkg.core.v1 as corev1

schema CafeDeploy:
    data: corev1.Deploy = corev1.Deploy {
        name: "deploy"
    }
