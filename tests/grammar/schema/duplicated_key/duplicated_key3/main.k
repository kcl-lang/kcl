schema Container:
    cpu?: int
    
schema App:
    container: Container

container = Container {cpu: 1}

app = App {
    container: container
    container: container
}