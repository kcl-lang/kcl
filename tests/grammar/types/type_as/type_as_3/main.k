schema Base:
    id?: int = 1

schema Sub(Base):
    data?: int = 2

sub: Base = Sub {}
id = (sub as Sub).id
