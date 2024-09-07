build-llvm:
	@echo "Building LLVM"
	mkdir -p build
	mkdir -p target/debug/deps
	mkdir -p target/release/deps
	g++ -c ./src/llvm/wrapping.cpp -o build/wrapping.o
	ar -rv target/debug/deps/libllvm.a build/wrapping.o
	cp target/debug/deps/libllvm.a target/release/deps/libllvm.a
