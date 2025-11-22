import .container_mixin as c_mixin

schema Container(c_mixin.BaseContainer):
    mixin [c_mixin.ContainerMixin]
    key?: str = None
