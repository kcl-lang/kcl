import ..apimachinery

schema PodTemplateSpec:
    metadata?: apimachinery.ObjectMeta
    spec?: PodSpec
