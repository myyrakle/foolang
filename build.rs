use std::path::PathBuf;

fn run_llvm_config() -> String {
    let output = std::process::Command::new("./llvm-project/build/bin/llvm-config")
        .arg("--cxxflags")
        .arg("--libs")
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8(output.stdout).unwrap();
    output
}

const BINDING_FILE_PATH: &str = "./src/llvm/bindings.rs";

fn exists_binding_file() -> bool {
    let path = PathBuf::from(BINDING_FILE_PATH);

    path.exists()
}

fn main() {
    // println!("cargo:rustc-link-search=native=/home/path/to/rust/proyect/folder/contain/file.a");

    // 라이브러리 경로 지정
    println!("cargo:rustc-link-search=/home/myyrakle/Codes/Rust/foolang/llvm-project/build/lib");

    // 라이브러리 링크
    println!("cargo:rustc-link-lib=LLVMCore");

    let current_dir = PathBuf::from(std::env::current_dir().unwrap());

    // 바인딩 파일이 존재하면 생성하지 않음
    if exists_binding_file() {
        return;
    }

    let header_base_path = current_dir.join("llvm-project/llvm/include");

    // bindgen 생성
    let mut bindings_builder = bindgen::Builder::default();

    let headers = vec![
        "llvm/Support/SourceMgr.h",
        "llvm/IR/LLVMContext.h",
        "llvm/IR/Module.h",
        "llvm/IRReader/IRReader.h",
        "llvm/Support/TargetSelect.h",
        "llvm/Target/TargetOptions.h",
        "llvm/IR/LegacyPassManager.h",
        "llvm/Support/FileSystem.h",
        "llvm/Support/CodeGen.h",
        "llvm/MC/TargetRegistry.h",
        "llvm/Target/TargetMachine.h",
        "llvm/TargetParser/Host.h",
    ];

    for header in headers {
        bindings_builder = bindings_builder.header(header_base_path.join(header).to_str().unwrap());
    }

    let llvm_config = run_llvm_config();

    //    -fno-rtti  -D_DEBUG -D_GLIBCXX_ASSERTIONS

    let flags = vec![
        "-I/home/myyrakle/Codes/Rust/foolang/llvm-project/llvm/include",
        "-I/home/myyrakle/Codes/Rust/foolang/llvm-project/build/include",
        "-std=c++17",
        "-fno-exceptions",
        "-funwind-tables",
        "-D_GNU_SOURCE",
        "-D__STDC_CONSTANT_MACROS",
        "-D__STDC_FORMAT_MACROS",
        "-D__STDC_LIMIT_MACROS",
        "-xc++",
    ];

    for flag in flags {
        //println!("@ {}", flag);
        bindings_builder = bindings_builder.clang_arg(flag);
    }

    let bindings = bindings_builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // 생성한 코드를 소스파일로 생성
    let out_path = PathBuf::from("./src/llvm");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
