extern crate bindgen;

fn generate_shim_bindings() {
    let builder = bindgen::Builder::new("src/ffi/cc/gbm_shim.c");
    match builder.generate() {
        Ok(b) => b.write_to_file("src/ffi/gbm_shim.rs").unwrap(),
        Err(e) => panic!(e)
    };
}

pub fn main() {
    generate_shim_bindings();
}
