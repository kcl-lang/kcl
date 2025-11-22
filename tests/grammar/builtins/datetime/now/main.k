import datetime

format = "%H:%M:%S"
result1: bool = datetime.validate(datetime.now(), "%a %b %d %H:%M:%S %Y")
result2: bool = datetime.validate(datetime.now(format), format)
