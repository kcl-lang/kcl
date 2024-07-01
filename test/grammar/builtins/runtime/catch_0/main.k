import runtime

# Get the panic message here
msg = runtime.catch(lambda {
    [][0] # panic here
})
