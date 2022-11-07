import base.pkg.kusion_kubernetes.api.rbac.v1 as rbacv1
import base.pkg.kusion_models.kube.frontend.common
import base.pkg.kusion_models.kube.mixins

schema ClusterRole(common.Metadata):
    """
    rules: [PolicyRule], default is Undefined, optional
        Rules holds all the PolicyRules for this ClusterRole
    aggregationRule: AggregationRule, default is Undefined, optional
        AggregationRule is an optional field that describes how to build the Rules for this ClusterRole. If AggregationRule is set, then the Rules are controller managed and direct changes to Rules will be stomped by the controller.
    """
    mixin [
        mixins.MetadataMixin
    ]

    namespace = Undefined

    rules?: [rbacv1.PolicyRule]

    aggregationRule?: rbacv1.AggregationRule

    kubernetes: rbacv1.ClusterRole = rbacv1.ClusterRole {
        metadata = metadata
        rules = rules
        aggregationRule = aggregationRule
    }

    assert not namespace, "namespace is not allowed in ClusterRole"
