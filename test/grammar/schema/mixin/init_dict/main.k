schema card:
    num: int

schema Person:
    mixin [cardMixin]
    card_ids: [{str:int}]
    cards: [card]

schema cardMixin:
    cards = card_ids

person = Person {
    "card_ids": [{"num": 123}]
}
