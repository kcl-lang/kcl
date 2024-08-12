schema Container:
    name: str
    name1: 
    
schema Config:
    mainContainer: Container
    image: str
    
schema Server[inputConfig: Config]:
    config: Config = 
    mainContainer: {str:}
    
    if 
