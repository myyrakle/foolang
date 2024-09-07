build-llvm:
	@echo "Building LLVM"
	mkdir -p build
	g++ -c ./src/llvm/wrapping.cpp -o build/wrapping.o
	ar -rv libllvm.a build/wrapping.o
