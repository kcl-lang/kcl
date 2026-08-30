schema ContainerInputs:
    maxReplicas?: int = 3

schema DeploymentInputs:
    [...str]: ContainerInputs

schema DeploymentCollectionInputs:
    [...str]: DeploymentInputs

schema NewContainerInputs(ContainerInputs):
    minReplicas?: int = 2

schema NewDeploymentInputs(DeploymentInputs):
    [...str]: NewContainerInputs

schema NewDeploymentCollectionInputs(DeploymentCollectionInputs):
    [...str]: NewDeploymentInputs

schema Inputs:
    vdeployments: DeploymentCollectionInputs

schema NewInputs(Inputs):
    vdeployments: NewDeploymentCollectionInputs

a = Inputs {
    vdeployments: {
        deploy_a: {
            container_a: {
                maxReplicas: 4
            }
        }
    }
}
b = NewInputs {
    vdeployments: {
        deploy_b: {
            container_b: {
              minReplicas: 2
            }
        }
    }
}