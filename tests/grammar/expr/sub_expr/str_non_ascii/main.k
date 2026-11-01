import runtime

s = "héllo"
a = s[1]
b = s[-1]
c = s[-4]
msg = runtime.catch(lambda {
    "héllo"[5] # out of range in characters, not in bytes
})
