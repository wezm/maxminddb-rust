use super::{MaxMindDBError, Reader};

use std::str::FromStr;
use std::net::IpAddr;

#[test]
fn test_decoder() {

    #[allow(non_snake_case)]
    #[derive(RustcDecodable, Debug, Eq, PartialEq)]
    struct MapXType {
        arrayX: Vec<usize>,
        utf8_stringX: String,
    };

    #[allow(non_snake_case)]
    #[derive(RustcDecodable, Debug, Eq, PartialEq)]
    struct MapType {
        mapX: MapXType,
    };

    #[derive(RustcDecodable, Debug)]
    struct TestType {
        array: Vec<usize>,
        boolean: bool,
        bytes: Vec<u8>,
        double: f64,
        float: f32,
        int32: i32,
        map: MapType,
        uint16: u16,
        uint32: u32,
        uint64: u64,
        uint128: Vec<u8>,
        utf8_string: String,
    }

    let r = Reader::open("test-data/test-data/MaxMind-DB-test-decoder.mmdb")
        .ok()
        .unwrap();
    let ip: IpAddr = FromStr::from_str("1.1.1.0").unwrap();
    let result: TestType = r.lookup(ip).unwrap();

    assert_eq!(result.array, vec![1usize, 2usize, 3usize]);
    assert_eq!(result.boolean, true);
    assert_eq!(result.bytes, vec![0u8, 0u8, 0u8, 42u8]);
    assert_eq!(result.double, 42.123456);
    assert_eq!(result.float, 1.1);
    assert_eq!(result.int32, -268435456);

    assert_eq!(result.map,
               MapType {
                   mapX: MapXType {
                       arrayX: vec![7, 8, 9],
                       utf8_stringX: "hello".to_string(),
                   },
               });

    assert_eq!(result.uint16, 100);
    assert_eq!(result.uint32, 268435456);
    assert_eq!(result.uint64, 1152921504606846976);
    assert_eq!(result.uint128,
               vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    assert_eq!(result.utf8_string, "unicode! ☯ - ♫".to_string());
}

#[test]
fn test_broken_database() {
    let r = Reader::open("test-data/test-data/GeoIP2-City-Test-Broken-Double-Format.mmdb")
        .ok()
        .unwrap();
    let ip: IpAddr = FromStr::from_str("2001:220::").unwrap();

    #[derive(RustcDecodable, Debug)]
    struct TestType;
    match r.lookup::<TestType>(ip) {
        Err(e) => {
            assert_eq!(e,
                       MaxMindDBError::InvalidDatabaseError("double of size 2".to_string()))
        }
        Ok(_) => panic!("Error expected"),
    }
}

#[test]
fn test_missing_database() {
    let r = Reader::open("file-does-not-exist.mmdb");
    match r {
        Ok(_) => panic!("Received Reader when opening non-existent file"),
        Err(MaxMindDBError::IoError(_)) => assert!(true),
        Err(_) => assert!(false),
    }
}

#[test]
fn test_non_database() {
    let r = Reader::open("README.md");
    match r {
        Ok(_) => panic!("Received Reader when opening a non-MMDB file"),
        Err(e) => {
            assert_eq!(e,
                       MaxMindDBError::InvalidDatabaseError("Could not find MaxMind DB metadata \
                                                             in file."
                                                                    .to_string()))
        }

    }
}

#[test]
fn test_reader() {
    let sizes = [24usize, 28, 32];
    for record_size in sizes.iter() {
        let versions = [4usize, 6];
        for ip_version in versions.iter() {
            let filename = format!("test-data/test-data/MaxMind-DB-test-ipv{}-{}.mmdb",
                                   ip_version,
                                   record_size);
            let reader = Reader::open(filename.as_ref()).ok().unwrap();

            check_metadata(&reader, *ip_version, *record_size);
            check_ip(&reader, *ip_version);
        }
    }
}

fn check_metadata(reader: &Reader, ip_version: usize, record_size: usize) {
    let metadata = &reader.metadata;

    assert_eq!(metadata.binary_format_major_version, 2u16);

    assert_eq!(metadata.binary_format_minor_version, 0u16);
    assert!(metadata.build_epoch >= 1397457605);
    assert_eq!(metadata.database_type, "Test".to_string());

    assert_eq!(*metadata.description.get(&"en".to_string()).unwrap(),
               "Test Database".to_string());
    assert_eq!(*metadata.description.get(&"zh".to_string()).unwrap(),
               "Test Database Chinese".to_string());

    assert_eq!(metadata.ip_version, ip_version as u16);
    assert_eq!(metadata.languages, vec!["en".to_string(), "zh".to_string()]);

    if ip_version == 4 {
        assert_eq!(metadata.node_count, 37)
    } else {
        assert_eq!(metadata.node_count, 160)
    }

    assert_eq!(metadata.record_size, record_size as u16)
}

fn check_ip(reader: &Reader, ip_version: usize) {

    let subnets = match ip_version {
        6 => {
            ["::1:ffff:ffff",
             "::2:0:0",
             "::2:0:0",
             "::2:0:0",
             "::2:0:0",
             "::2:0:40",
             "::2:0:40",
             "::2:0:40",
             "::2:0:50",
             "::2:0:50",
             "::2:0:50",
             "::2:0:58",
             "::2:0:58"]
        }
        _ => {
            ["1.1.1.1", "1.1.1.2", "1.1.1.2", "1.1.1.4", "1.1.1.4", "1.1.1.4", "1.1.1.4",
             "1.1.1.8", "1.1.1.8", "1.1.1.8", "1.1.1.16", "1.1.1.16", "1.1.1.16"]
        }
    };

    #[derive(RustcDecodable, Debug)]
    struct IpType {
        ip: String,
    }

    for subnet in subnets.iter() {
        let ip: IpAddr = FromStr::from_str(&subnet).unwrap();
        let value: IpType = reader.lookup(ip).unwrap();

        assert_eq!(value.ip, subnet.to_string());
    }

    let no_record = ["1.1.1.33", "255.254.253.123", "89fa::"];

    for &address in no_record.iter() {
        let ip: IpAddr = FromStr::from_str(address).unwrap();
        match reader.lookup::<IpType>(ip) {
            Ok(v) => panic!("received an unexpected value: {:?}", v),
            Err(e) => {
                assert_eq!(e,
                           MaxMindDBError::AddressNotFoundError("Address not found in database"
                                                                    .to_string()))
            }
        }
    }
}
