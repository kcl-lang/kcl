
K_FILES=$(wildcard ./*.k)
K_JSON_FILES=$(patsubst %.k,%.k.json,$(K_FILES))

default: clean $(K_JSON_FILES) $(K_FMT_JSON_FILES)
	@echo "ok"

clean:
	-rm *.k.json

%.k.json: %.k
	cargo run --bin parse_file $< > $@

