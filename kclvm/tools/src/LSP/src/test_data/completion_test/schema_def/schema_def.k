schema Container:
    name: str
   
    
schema Config:
    mainContainer: Container
    image: str
    
schema Server[inputConfig: Config]:
    config: Config = 
    mainContainer: {str:}
    
    if 
