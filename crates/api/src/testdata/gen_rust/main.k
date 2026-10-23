"""Customer domain DTOs."""
import regex

type Currency = "EUR" | "USD" | "GBP"

schema Entity:
    """Base for all persisted entities."""
    id: str
    version: int = 0

    check:
        regex.match(id, r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"), "id must be a UUID"
        version >= 0

schema Address:
    """Postal address value object."""
    street: str
    city: str
    country: str

    check:
        len(country) == 2, "country must be an ISO 3166-1 alpha-2 code"
        country == country.upper()

schema Customer(Entity):
    """Customer DTO. Extends Entity."""
    name: str
    email: str
    age?: int
    credit_limit: float = 0
    currency: Currency = "EUR"
    tags: [str] = []
    addresses: [Address] = []
    billing_address?: Address

    check:
        1 <= len(name) <= 100, "name must be 1..100 chars"
        regex.match(email, r"^[^@\s]+@[^@\s]+\.[^@\s]+$"), "invalid email"
        age >= 18 if age, "customer must be an adult"
        0 <= credit_limit <= 1000000
        len(tags) <= 10
        all t in tags {len(t) > 0}
        billing_address in addresses if billing_address, "billing_address must be one of addresses"