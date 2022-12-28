use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::PathBuf;

use crate::iso::directory_entry::DirectoryEntry;

use crate::iso::file_entry::FileEntry;
use crate::iso::utils::{LOGIC_SIZE, LOGIC_SIZE_U32};
use crate::iso::volume_descriptor::VolumeDescriptor;

#[macro_use]
pub mod utils;
mod directory_entry;
pub mod file_entry;
mod volume_descriptor;

fn assign_directory_identifiers(
    tree: &mut DirectoryEntry,
    last_index: &mut u32,
    last_lba: &mut u32,
) {
    // Reserve CE space for SUSP
    if tree.continuation_area.is_some() {
        *last_lba += 1;
    }

    if *last_index == 0 {
        tree.parent_index = *last_index;
        tree.path_table_index = *last_index + 1;

        *last_index = tree.path_table_index;
    } else {
        tree.lba = *last_lba;
    }
    *last_lba += tree.get_extent_size_in_lb();
}

fn reserve_file_space(directory_entry: &mut DirectoryEntry, current_lba: &mut u32) {
    for child_file in &mut directory_entry.files_childs {
        let lba_count = ((child_file.size as u32) + LOGIC_SIZE_U32) / LOGIC_SIZE_U32;
        child_file.lba = *current_lba;
        *current_lba += lba_count;
    }
}

fn write_system_area<T>(
    _tree: &mut DirectoryEntry,
    output_writter: &mut T,
    _lb_count: u32,
) -> std::io::Result<()>
where
    T: Write + Seek,
{
    let old_pos = output_writter.seek(SeekFrom::Current(0))?;

    let current_pos = output_writter.seek(SeekFrom::Current(0))?;

    // Pad to 0x8000 if needed
    let diff_size = current_pos as usize - old_pos as usize;

    if diff_size != LOGIC_SIZE * 0x10 {
        let mut padding: Vec<u8> = Vec::new();
        padding.resize(LOGIC_SIZE * 0x10 - diff_size, 0u8);
        output_writter.write_all(&padding)?;
    }

    Ok(())
}

pub fn create_iso(output: String, entries: Vec<FileEntry>) -> std::io::Result<()> {
    let mut volume_descriptor_list = Vec::new();

    volume_descriptor_list.push(VolumeDescriptor::Primary);
    volume_descriptor_list.push(VolumeDescriptor::End);

    let mut out_file = File::create(&output)?;

    let mut current_lba: u32 = 0x10 + 1 + (volume_descriptor_list.len() as u32);

    let path_table_start_lba = current_lba;

    // Reserve 4 LBA for path tables (add some spacing after table)
    current_lba += 4;

    let mut tree = DirectoryEntry::new()?;

    tree.set_path(entries)?;
    let mut path_table_index = 0;

    let mut tmp_lba = current_lba;

    // create 'ER' entry of Rock Ridge 1.2
    let mut continuation_area: Vec<u8> = Vec::new();
    continuation_area.write_all(b"ER")?;
    continuation_area.write_u8(0xB6)?;
    continuation_area.write_u8(0x1)?;
    continuation_area.write_u8(0x9)?;
    continuation_area.write_u8(0x48)?;
    continuation_area.write_u8(0x5d)?;
    continuation_area.write_u8(0x1)?;
    continuation_area.write_all(b"IEEE_1282")?;
    continuation_area
        .write_all(b"THE IEEE 1282 PROTOCOL PROVIDES SUPPORT FOR POSIX FILE SYSTEM SEMANTICS.")?;
    continuation_area.write_all(b"PLEASE CONTACT THE IEEE STANDARDS DEPARTMENT, PISCATAWAY, NJ, USA FOR THE 1282 SPECIFICATION.")?;
    tree.continuation_area = Some(continuation_area);

    assign_directory_identifiers(&mut tree, &mut path_table_index, &mut tmp_lba);
    tree.parent_index = 1;
    tree.lba = current_lba;

    current_lba = tmp_lba;
    current_lba += 1;

    reserve_file_space(&mut tree, &mut current_lba);

    write_system_area(&mut tree, &mut out_file, current_lba)?;

    for mut volume in volume_descriptor_list {
        volume.write_volume(&mut out_file, &mut tree, path_table_start_lba, current_lba)?;
    }

    // FIXME: what is this and why do I need it???? checksum infos??
    let empty_mki_section: [u8; 2044] = [0; 2044];
    out_file.write_all(b"MKI ")?;
    out_file.write_all(&empty_mki_section)?;

    tree.write_path_table::<File, LittleEndian>(&mut out_file, path_table_start_lba)?;
    tree.write_path_table::<File, BigEndian>(&mut out_file, path_table_start_lba + 1)?;
    tree.write_extent(&mut out_file, None)?;
    tree.write_files(&mut out_file)?;

    Ok(())
}
