fn main() {
    println!("cargo:rustc-link-search=native=/home/path/to/rust/proyect/folder/contain/file.a");
    println!("cargo:rustc-link-lib=static=test");
}
