import base.pkg.kusion_kubernetes.api.rbac.v1 as rbacv1
import base.pkg.kusion_models.kube.frontend.common
import base.pkg.kusion_models.kube.mixins

schema Role(common.Metadata):
    """
    rules: [PolicyRule], default is Undefined, optional
        Rules holds all the PolicyRules for this ClusterRole
    """
    mixin [
        mixins.MetadataMixin
    ]

    rules?: [rbacv1.PolicyRule]

    kubernetes: rbacv1.Role = rbacv1.Role {
        metadata = metadata
        rules = rules
    }
