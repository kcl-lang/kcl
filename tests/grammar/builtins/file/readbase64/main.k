import file

# A small binary file: the base64 encoding of the raw bytes should be returned.
binary_b64 = file.readbase64("sample.bin")

# A text file: readbase64 should still work and return the base64 of the bytes.
text_b64 = file.readbase64("hello.txt")
