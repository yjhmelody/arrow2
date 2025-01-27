use std::collections::VecDeque;
use std::io::{Read, Seek};

use gen::Schema::MetadataVersion;

use crate::array::UnionArray;
use crate::datatypes::DataType;
use crate::error::Result;
use crate::io::ipc::gen::Message::BodyCompression;

use super::super::super::gen;
use super::super::deserialize::{read, skip, Node};
use super::super::read_basic::*;

#[allow(clippy::too_many_arguments)]
pub fn read_union<R: Read + Seek>(
    field_nodes: &mut VecDeque<Node>,
    data_type: DataType,
    buffers: &mut VecDeque<&gen::Schema::Buffer>,
    reader: &mut R,
    block_offset: u64,
    is_little_endian: bool,
    compression: Option<BodyCompression>,
    version: MetadataVersion,
) -> Result<UnionArray> {
    let field_node = field_nodes.pop_front().unwrap().0;

    if version != MetadataVersion::V5 {
        let _ = buffers.pop_front().unwrap();
    };

    let types = read_buffer(
        buffers,
        field_node.length() as usize,
        reader,
        block_offset,
        is_little_endian,
        compression,
    )?;

    let offsets = if let DataType::Union(_, _, is_sparse) = data_type {
        if !is_sparse {
            Some(read_buffer(
                buffers,
                field_node.length() as usize,
                reader,
                block_offset,
                is_little_endian,
                compression,
            )?)
        } else {
            None
        }
    } else {
        panic!()
    };

    let fields = UnionArray::get_fields(&data_type);

    let fields = fields
        .iter()
        .map(|field| {
            read(
                field_nodes,
                field.data_type().clone(),
                buffers,
                reader,
                block_offset,
                is_little_endian,
                compression,
                version,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(UnionArray::from_data(data_type, types, fields, offsets))
}

pub fn skip_union(
    field_nodes: &mut VecDeque<Node>,
    data_type: &DataType,
    buffers: &mut VecDeque<&gen::Schema::Buffer>,
) {
    let _ = field_nodes.pop_front().unwrap();

    let _ = buffers.pop_front().unwrap();
    if let DataType::Union(_, _, is_sparse) = data_type {
        if !*is_sparse {
            let _ = buffers.pop_front().unwrap();
        }
    } else {
        panic!()
    };

    let fields = UnionArray::get_fields(data_type);

    fields
        .iter()
        .for_each(|field| skip(field_nodes, field.data_type(), buffers))
}
