use spirv_builder::{MetadataPrintout, SpirvBuilder, Capability};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("../voxel-engine-shader", "spirv-unknown-opengl4.5")
        .print_metadata(MetadataPrintout::Full)
        .capability(Capability::ImageQuery)
        .build()?;
    Ok(())
}
