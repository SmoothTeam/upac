// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use tar::{Builder as TarBuilder, Header as TarHeader};
use tempfile::TempDir;
use xz2::write::XzEncoder;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use super::SourceArchive;

fn write_plain_tar(path: &Path, entry_name: &str, content: &[u8]) {
    let file = File::create(path).unwrap();
    let mut builder = TarBuilder::new(file);

    let mut header = TarHeader::new_gnu();
    header.set_size(content.len() as u64);
    header.set_cksum();
    builder.append_data(&mut header, entry_name, content).unwrap();
    builder.finish().unwrap();
}

fn assert_round_trip(archive_path: &Path, entry_name: &str, content: &[u8]) {
    let archive = SourceArchive::sniff(archive_path).unwrap();

    let destination = TempDir::new().unwrap();
    archive.extract(destination.path()).unwrap();

    let extracted = fs::read(destination.path().join(entry_name)).unwrap();
    assert_eq!(extracted, content);
}

#[test]
fn sniff_and_extract_round_trip_plain_tar() {
    let scratch = TempDir::new().unwrap();
    let archive_path = scratch.path().join("source.tar");
    write_plain_tar(&archive_path, "hello.txt", b"hello from plain tar");

    assert_round_trip(&archive_path, "hello.txt", b"hello from plain tar");
}

#[test]
fn sniff_and_extract_round_trip_gzip_tar() {
    let scratch = TempDir::new().unwrap();
    let tar_path = scratch.path().join("source.tar");
    write_plain_tar(&tar_path, "hello.txt", b"hello from gzip tar");
    let tar_bytes = fs::read(&tar_path).unwrap();

    let archive_path = scratch.path().join("source.tar.gz");
    let out = File::create(&archive_path).unwrap();
    let mut encoder = flate2::write::GzEncoder::new(out, flate2::Compression::default());
    encoder.write_all(&tar_bytes).unwrap();
    encoder.finish().unwrap();

    assert_round_trip(&archive_path, "hello.txt", b"hello from gzip tar");
}

#[test]
fn sniff_and_extract_round_trip_xz_tar() {
    let scratch = TempDir::new().unwrap();
    let tar_path = scratch.path().join("source.tar");
    write_plain_tar(&tar_path, "hello.txt", b"hello from xz tar");
    let tar_bytes = fs::read(&tar_path).unwrap();

    let archive_path = scratch.path().join("source.tar.xz");
    let out = File::create(&archive_path).unwrap();
    let mut encoder = XzEncoder::new(out, 6);
    encoder.write_all(&tar_bytes).unwrap();
    encoder.finish().unwrap();

    assert_round_trip(&archive_path, "hello.txt", b"hello from xz tar");
}

#[test]
fn sniff_and_extract_round_trip_zstd_tar() {
    let scratch = TempDir::new().unwrap();
    let tar_path = scratch.path().join("source.tar");
    write_plain_tar(&tar_path, "hello.txt", b"hello from zstd tar");
    let tar_bytes = fs::read(&tar_path).unwrap();

    let archive_path = scratch.path().join("source.tar.zst");
    let compressed = zstd::encode_all(&tar_bytes[..], 0).unwrap();
    fs::write(&archive_path, compressed).unwrap();

    assert_round_trip(&archive_path, "hello.txt", b"hello from zstd tar");
}

#[test]
fn sniff_and_extract_round_trip_zip() {
    let scratch = TempDir::new().unwrap();
    let archive_path = scratch.path().join("source.zip");

    let file = File::create(&archive_path).unwrap();
    let mut writer = ZipWriter::new(file);
    writer.start_file("hello.txt", SimpleFileOptions::default()).unwrap();
    writer.write_all(b"hello from zip").unwrap();
    writer.finish().unwrap();

    assert_round_trip(&archive_path, "hello.txt", b"hello from zip");
}

#[test]
fn sniff_detects_sevenzip_magic_without_a_full_archive() {
    let scratch = TempDir::new().unwrap();
    let archive_path = scratch.path().join("source.7z");
    fs::write(&archive_path, [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 0x00, 0x04]).unwrap();

    let archive = SourceArchive::sniff(&archive_path).unwrap();

    assert!(matches!(archive, SourceArchive::SevenZip(_)));
}
