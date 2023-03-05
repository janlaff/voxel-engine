use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("../voxel-engine-shader", "spirv-unknown-opengl4.5")
        .print_metadata(MetadataPrintout::Full)
        .capability(Capability::ImageQuery)
        .build()?;
    Ok(())
}
