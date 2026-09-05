// SPDX-FileCopyrightText: 2026 JustPav
// SPDX-FileCopyrightText: 2026 SmoothTeam
//
// SPDX-License-Identifier: LGPL-3.0-or-later WITH LGPL-3.0-linking-exception

use upac_types::codec::{RedbCodable, write_len_prefixed, write_opt_str};

#[test]
fn bool_redb_round_trip() {
    for value in [true, false] {
        let mut buf = Vec::new();
        value.redb_encode(&mut buf);

        let mut offset = 0;
        assert_eq!(bool::redb_decode(&buf, &mut offset), value);
        assert_eq!(offset, buf.len());
    }
}

#[test]
fn u32_redb_round_trip() {
    let mut buf = Vec::new();
    42u32.redb_encode(&mut buf);

    let mut offset = 0;
    assert_eq!(u32::redb_decode(&buf, &mut offset), 42);
    assert_eq!(offset, buf.len());
}

#[test]
fn u64_redb_round_trip() {
    let mut buf = Vec::new();
    123_456_789u64.redb_encode(&mut buf);

    let mut offset = 0;
    assert_eq!(u64::redb_decode(&buf, &mut offset), 123_456_789);
    assert_eq!(offset, buf.len());
}

#[test]
fn string_redb_round_trip() {
    let mut buf = Vec::new();
    "hello".to_owned().redb_encode(&mut buf);

    let mut offset = 0;
    assert_eq!(String::redb_decode(&buf, &mut offset), "hello");
    assert_eq!(offset, buf.len());
}

#[test]
fn option_redb_round_trip() {
    let mut buf = Vec::new();
    Some(7u32).redb_encode(&mut buf);
    None::<u32>.redb_encode(&mut buf);

    let mut offset = 0;
    assert_eq!(Option::<u32>::redb_decode(&buf, &mut offset), Some(7));
    assert_eq!(Option::<u32>::redb_decode(&buf, &mut offset), None);
    assert_eq!(offset, buf.len());
}

#[test]
fn vec_redb_round_trip() {
    let mut buf = Vec::new();
    vec![1u32, 2, 3].redb_encode(&mut buf);

    let mut offset = 0;
    assert_eq!(Vec::<u32>::redb_decode(&buf, &mut offset), vec![1, 2, 3]);
    assert_eq!(offset, buf.len());
}

#[test]
fn write_len_prefixed_round_trips_through_string_decode() {
    let mut buf = Vec::new();
    write_len_prefixed(&mut buf, b"upac");

    let mut offset = 0;
    assert_eq!(String::redb_decode(&buf, &mut offset), "upac");
    assert_eq!(offset, buf.len());
}

#[test]
fn write_opt_str_some_round_trips_through_option_string_decode() {
    let mut buf = Vec::new();
    write_opt_str(&mut buf, Some("v3"));

    let mut offset = 0;
    assert_eq!(Option::<String>::redb_decode(&buf, &mut offset), Some("v3".to_owned()));
    assert_eq!(offset, buf.len());
}

#[test]
fn write_opt_str_none_round_trips_through_option_string_decode() {
    let mut buf = Vec::new();
    write_opt_str(&mut buf, None);

    let mut offset = 0;
    assert_eq!(Option::<String>::redb_decode(&buf, &mut offset), None);
    assert_eq!(offset, buf.len());
}
