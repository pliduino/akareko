use skerry_codegen::SkerryGenerator;

fn main() -> Result<(), skerry_codegen::SkerryCodeGenError> {
    SkerryGenerator::new().generate()
}
