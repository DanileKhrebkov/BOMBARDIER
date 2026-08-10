fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/hello.proto");
    
    tonic_build::compile_protos("proto/hello.proto")?;
    Ok(())
}
