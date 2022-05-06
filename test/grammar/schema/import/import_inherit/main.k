import pkg

schema Nameable(pkg.Name):
	id: int

myPlant = Nameable {
    "firstName": "seed",
    "lastName": "green",
    "id": 1
}