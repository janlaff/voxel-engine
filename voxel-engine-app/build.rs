use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Generate voxel-engine-gpu SPIR-V shader
    SpirvBuilder::new("../voxel-engine-shader", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .capability(Capability::ImageQuery)
        .capability(Capability::Int8)
        .capability(Capability::Int16)
        .build()?;

    // Generate build info
    built::write_built_file().expect("Failed to write built file");

    Ok(())
}
