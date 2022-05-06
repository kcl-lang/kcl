# Units based on SI and ECI
# Ref: https://en.wikipedia.org/wiki/Binary_prefix
# Kubernetes use case: https://pkg.go.dev/k8s.io/apimachinery/pkg/api/resource#Quantity

from typing import Union


IEC_SUFFIX = "i"
EXPONENTS = {"n": -3, "u": -2, "m": -1, "K": 1, "k": 1, "M": 2, "G": 3, "T": 4, "P": 5}
NUMBER_MULTIPLIER_REGEX = r"^([1-9][0-9]{0,63})(E|P|T|G|M|K|k|m|u|n|Ei|Pi|Ti|Gi|Mi|Ki)$"


def cal_num(value: int, suffix: str) -> int:
    """
    Calculate number based on value and binary suffix.

    Supported suffixes:
    SI: n | u | m | k | K | M | G | T | P
    IEC: Ki | Mi | Gi | Ti | Pi

    Input:
    value: int.
    suffix: str.

    Returns:
    int

    Raises:
    ValueError on invalid or unknown suffix
    """

    if not isinstance(value, int):
        raise ValueError("Unsupported value type: {}".format(type(value)))

    if not suffix:
        return value

    base = 1000
    unit = suffix

    validate_unit(unit)

    if unit[-1] == "i":
        base = 1024
        unit = unit[:-1]

    exponent = EXPONENTS[unit]
    return value * (base ** exponent)


def to_quantity(quantity: Union[str, int]) -> int:
    """
    Parse and return number based on input quantity.

    Supported suffixes:
    SI: n | u | m | k | K | M | G | T | P
    IEC: Ki | Mi | Gi | Ti | Pi

    Input:
    quantity: str.

    Returns:
    int

    Raises:
    ValueError on invalid or unknown input
    """
    if not isinstance(quantity, (int, str)):
        raise ValueError("Unsupported quantity type: {}".format(type(quantity)))

    if isinstance(quantity, int):
        return quantity

    number = quantity
    suffix = None
    if len(quantity) >= 2 and quantity[-1] == IEC_SUFFIX:
        if quantity[-2] in EXPONENTS:
            number = quantity[:-2]
            suffix = quantity[-2:]
    elif len(quantity) >= 1 and quantity[-1] in EXPONENTS:
        number = quantity[:-1]
        suffix = quantity[-1:]

    if not number:
        raise ValueError("Number can't be empty")

    number = int(number)

    if suffix is None:
        return number

    validate_unit(suffix[0])

    if suffix.endswith(IEC_SUFFIX):
        base = 1024
    else:
        base = 1000

    exponent = EXPONENTS[suffix[0]]
    return number * (base ** exponent)


def validate_unit(unit: str) -> None:
    # IEC validate
    if not unit or not isinstance(unit, str) or len(unit) > 2:
        raise ValueError("Invalid suffix {}".format(unit))

    if unit in ["ni", "ui", "mi", "ki"]:
        raise ValueError("Invalid suffix {}".format(unit))

    if unit[0] not in EXPONENTS:
        raise ValueError("Invalid suffix {}".format(unit))
