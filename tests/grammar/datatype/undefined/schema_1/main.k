schema Base:
    id?: int = 1
    name?: str = None
    labels?: {str:} = {"key": "value"}

schema Sub(Base):
    id?: int = Undefined

base = Base {}
sub = Sub {}
