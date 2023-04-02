use nom::bytes::complete::{tag, take};
use nom::combinator::{map, map_res};
use nom::number::complete::{be_i32, be_u32, le_i32, le_u32};
use nom::sequence::{pair, preceded, tuple};
use nom::IResult;

#[derive(Debug, Clone, Copy)]
struct ChunkHeader<'a> {
    id: &'a str,
    num_content_bytes: i32,
    num_children_bytes: i32,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Voxel {
    x: u32,
    y: u32,
    z: u32,
    color_index: u32,
}

#[derive(Debug)]
pub enum ChunkContent {
    None,
    Size(i32, i32, i32),
    Xyzi(Vec<Voxel>),
}

fn parse_chunk_header<'a>(input: &'a [u8]) -> IResult<&[u8], ChunkHeader<'a>> {
    map(
        tuple((take(4usize), le_i32, le_i32)),
        |(id, num_content_bytes, num_children_bytes)| ChunkHeader {
            id: std::str::from_utf8(id).unwrap(),
            num_content_bytes,
            num_children_bytes,
        },
    )(input)
}

fn parse_size_chunk_content(input: &[u8]) -> IResult<&[u8], ChunkContent> {
    map(tuple((le_i32, le_i32, le_i32)), |(x, y, z)| {
        ChunkContent::Size(x, y, z)
    })(input)
}

fn parse_xyzi_chunk_content(input: &[u8]) -> IResult<&[u8], ChunkContent> {
    let mut voxels = Vec::new();

    let (mut input, num_voxel_bytes) = le_i32(input)?;

    let mut voxel = Voxel::default();
    for i in 0..(num_voxel_bytes / 4) {
        (input, voxel) = map(
            tuple((le_u32, le_u32, le_u32, le_u32)),
            |(x, y, z, color_index)| Voxel {
                x,
                y,
                z,
                color_index,
            },
        )(input)?;

        voxels.push(voxel);
    }

    Ok((input, ChunkContent::Xyzi(voxels)))
}

pub fn parse_vox(input: &[u8]) -> IResult<&[u8], Vec<ChunkContent>> {
    let (input, version) = preceded(tag("VOX "), le_i32)(input)?;

    if version != 150 {
        panic!("Unsupported vox version");
    }

    // Parse main chunk
    let (mut input, mut chunk_header) = parse_chunk_header(input)?;
    if chunk_header.id != "MAIN" {
        panic!("No main chunk found");
    }

    let mut chunk_contents = Vec::new();
    let mut chunk_content = None;

    while !input.is_empty() {
        (input, chunk_header) = parse_chunk_header(input)?;
        (input, chunk_content) = match chunk_header.id {
            "SIZE" => map(parse_size_chunk_content, |res| Some(res))(input)?,
            "XYZI" => map(parse_xyzi_chunk_content, |res| Some(res))(input)?,
            _ => {
                println!("Drop chunk {}", chunk_header.id);
                (&input[chunk_header.num_content_bytes as usize..], None)
            }
        };

        if let Some(content) = chunk_content {
            println!("Load chunk {}", chunk_header.id);
            chunk_contents.push(content);
        }
    }

    Ok((input, chunk_contents))
}

#[test]
pub fn test_parser() {
    let model = include_bytes!("vox/menger.vox");
    parse_vox(model).unwrap();
}
