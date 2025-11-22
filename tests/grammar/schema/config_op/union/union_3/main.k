schema ResourceMapping:
    [str]: any

schema BaseFrontend:
    name: str

protocol MixinProtocol:
    count: int
    resources: ResourceMapping

mixin Mixin1Mixin for MixinProtocol:
    resources: ResourceMapping {
        resource1 += [{
            a = "a"
        }]
    }
    count += 1

mixin Mixin2Mixin for MixinProtocol:
    resources: ResourceMapping {
        resource2 += [{
            b = "b"
        }]
    }
    count += 1

mixin Mixin3Mixin for MixinProtocol:
    resources: ResourceMapping {
        resource3 += [{
            c = "c"
        }]
    }
    count += 1

schema BaseBackend[a: BaseFrontend]:
    mixin [
        Mixin1Mixin,
        Mixin2Mixin,
        Mixin3Mixin,
        Mixin3Mixin,
    ]
    resources: ResourceMapping
    count: int = 0

render = lambda a: BaseFrontend {
    impl = BaseBackend(a)
    impl
}
frontEnd = BaseFrontend {name = "app"}
res1 = [render(a) for a in BaseFrontend.instances()]
res2 = [BaseBackend(a) for a in BaseFrontend.instances()]
resources: ResourceMapping {}
resources: ResourceMapping {
    resource1 += [{
        a = "a"
    }]
}
resources: ResourceMapping {
    resource2 += [{
        b = "b"
    }]
}
