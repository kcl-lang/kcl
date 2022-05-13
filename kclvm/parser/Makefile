TARG:=hello.k

IGNORED_KEYS:=line,column,end_line,end_column,comments,filename

dev:
	-@cargo fmt

	cargo build
	./target/debug/parse_module ${TARG} 2>&1 1> ./a.out.json

	cat ./a.out.json | jq

	-@rm ./a.out.json

debug:
	RUST_BACKTRACE=1 cargo run --bin parse_module ${TARG}

diff:
	-@ rm *.out.json
	cargo run --bin load_program ${TARG} | jq -S > 1.out.json
	kclvm -m kclvm.internal.kclx -f=${TARG} | jq -S > 0.out.json
	code --diff 0.out.json 1.out.json

ast:
	@kclvm -m kclvm.internal.kclx -f=${TARG} | jq

lint:
	cargo clippy

clean:
	cargo clean
