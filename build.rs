fn main() {
    prost_build::compile_protos(&["proto/syntaxdot.proto"], &["proto/"]).unwrap();
}
