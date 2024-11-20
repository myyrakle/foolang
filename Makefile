build-llvm:
	@cd llvm-project
	@cmake -S llvm -B build -G "Ninja" -DCMAKE_BUILD_TYPE=Debug
	@cd build
	@ninja -j2


