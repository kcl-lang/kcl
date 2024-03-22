schema MyApp:
   name: str
   versions: {str:str}
   version = versions[name]
