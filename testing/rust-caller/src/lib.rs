// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

extern crate ocaml_interop;

use ocaml_interop::{
    ocaml_alloc, ocaml_call, ocaml_frame, IntoRust, OCaml, OCamlBytes, OCamlInt, OCamlList, ToOCaml,
    to_ocaml,
};

mod ocaml {
    use ocaml_interop::{impl_to_ocaml_record, impl_to_ocaml_variant, ocaml, OCamlInt, OCamlInt32, OCamlInt64, OCamlList};

    pub struct TestRecord {
        pub i: i64,
        pub f: f64,
        pub i32: i32,
        pub i64: Box<i64>,
        pub s: String,
        pub t: (i64, f64),
    }

    pub enum Movement {
        Step(i64),
        RotateLeft,
        RotateRight,
    }

    impl_to_ocaml_record! {
        TestRecord {
            i: OCamlInt,
            f: f64,
            i32: OCamlInt32,
            i64: OCamlInt64,
            s: String,
            t: (OCamlInt, f64),
        }
    }

    impl_to_ocaml_variant! {
        Movement {
            Movement::Step(count: OCamlInt),
            Movement::RotateLeft,
            Movement::RotateRight,
        }
    }

    ocaml! {
        pub fn increment_bytes(bytes: String, first_n: OCamlInt) -> String;
        pub fn increment_ints_list(ints: OCamlList<OCamlInt>) -> OCamlList<OCamlInt>;
        pub fn twice(num: OCamlInt) -> OCamlInt;
        pub fn make_tuple(fst: String, snd: OCamlInt) -> (String, OCamlInt);
        pub fn make_some(value: String) -> Option<String>;
        pub fn verify_record(record: TestRecord) -> bool;
        pub fn stringify_variant(variant: Movement) -> String;
    }
}

pub fn increment_bytes(bytes: &str, first_n: usize) -> String {
    ocaml_frame!(gc, {
        let bytes = ocaml_alloc!(bytes.to_ocaml(gc));
        let bytes_ref = &gc.keep(bytes);
        let first_n = ocaml_alloc!((first_n as i64).to_ocaml(gc));
        let result = ocaml_call!(ocaml::increment_bytes(gc, gc.get(bytes_ref), first_n));
        let result: OCaml<String> = result.expect("Error in 'increment_bytes' call result");
        result.into_rust()
    })
}

pub fn increment_ints_list(ints: &Vec<i64>) -> Vec<i64> {
    ocaml_frame!(gc nokeep, {
        let ints = ocaml_alloc!(ints.to_ocaml(gc));
        let result = ocaml_call!(ocaml::increment_ints_list(gc, ints));
        let result: OCaml<OCamlList<OCamlInt>> =
            result.expect("Error in 'increment_ints_list' call result");
        result.into_rust()
    })
}

pub fn twice(num: i64) -> i64 {
    ocaml_frame!(gc nokeep, {
        let num = unsafe { OCaml::of_i64(num) };
        let result = ocaml_call!(ocaml::twice(gc, num));
        let result: OCaml<OCamlInt> = result.expect("Error in 'twice' call result");
        result.into_rust()
    })
}

pub fn make_tuple(fst: String, snd: i64) -> (String, i64) {
    ocaml_frame!(gc nokeep, {
        let num = unsafe { OCaml::of_i64(snd) };
        let str = ocaml_alloc!(fst.to_ocaml(gc));
        let result = ocaml_call!(ocaml::make_tuple(gc, str, num));
        let result: OCaml<(String, OCamlInt)> = result.expect("Error in 'make_tuple' call result");
        result.into_rust()
    })
}

pub fn make_some(value: String) -> Option<String> {
    ocaml_frame!(gc nokeep, {
        let str = ocaml_alloc!(value.to_ocaml(gc));
        let result = ocaml_call!(ocaml::make_some(gc, str));
        let result: OCaml<Option<String>> = result.expect("Error in 'make_some' call result");
        result.into_rust()
    })
}

pub fn verify_record_test(record: ocaml::TestRecord) -> bool {
    ocaml_frame!(gc, {
        let ocaml_record = ocaml_alloc!(record.to_ocaml(gc));
        let result = ocaml_call!(ocaml::verify_record(gc, ocaml_record));
        let result: OCaml<bool> = result.expect("Error in 'verify_record' call result");
        result.into_rust()
    })
}

pub fn verify_variant_test(variant: ocaml::Movement) -> String {
    ocaml_frame!(gc, {
        let ocaml_variant = to_ocaml!(gc, variant);
        let result = ocaml_call!(ocaml::stringify_variant(gc, ocaml_variant));
        let result: OCaml<String> = result.expect("Error in 'stringify_variant' call result");
        result.into_rust()
    })
}

pub fn allocate_alot() -> bool {
    let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    for _n in 1..50000 {
        ocaml_frame!(gc, {
            let _x: OCaml<OCamlBytes> = ocaml_alloc!(vec.to_ocaml(gc));
            let _y: OCaml<OCamlBytes> = ocaml_alloc!(vec.to_ocaml(gc));
            let _z: OCaml<OCamlBytes> = ocaml_alloc!(vec.to_ocaml(gc));
            ()
        });
    }
    true
}

// Tests

// NOTE: required because at the moment, no synchronization is done on OCaml calls
#[cfg(test)]
use serial_test::serial;

#[test]
#[serial]
fn test_twice() {
    ocaml_interop::OCamlRuntime::init_persistent();
    assert_eq!(twice(10), 20);
}

#[test]
#[serial]
fn test_increment_bytes() {
    ocaml_interop::OCamlRuntime::init_persistent();
    assert_eq!(increment_bytes("0000000000000000", 10), "1111111111000000");
}

#[test]
#[serial]
fn test_increment_ints_list() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let ints = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(increment_ints_list(&ints), expected);
}

#[test]
#[serial]
fn test_make_tuple() {
    ocaml_interop::OCamlRuntime::init_persistent();
    assert_eq!(make_tuple("fst".to_owned(), 9), ("fst".to_owned(), 9));
}

#[test]
#[serial]
fn test_make_some() {
    ocaml_interop::OCamlRuntime::init_persistent();
    assert_eq!(make_some("some".to_owned()), Some("some".to_owned()));
}

#[test]
#[serial]
fn test_frame_management() {
    ocaml_interop::OCamlRuntime::init_persistent();
    assert_eq!(allocate_alot(), true);
}

#[test]
#[serial]
fn test_record_conversion() {
    ocaml_interop::OCamlRuntime::init_persistent();
    let record = ocaml::TestRecord {
        i: 10,
        f: 5.0,
        i32: 10,
        i64: Box::new(10),
        s: "string".to_owned(),
        t: (10, 5.0),
    };
    assert_eq!(verify_record_test(record), true);
}

#[test]
#[serial]
fn test_variant_conversion() {
    ocaml_interop::OCamlRuntime::init_persistent();
    assert_eq!(verify_variant_test(ocaml::Movement::RotateLeft), "RotateLeft".to_owned());
    assert_eq!(verify_variant_test(ocaml::Movement::RotateRight), "RotateRight".to_owned());
    assert_eq!(verify_variant_test(ocaml::Movement::Step(10)), "Step(10)".to_owned());
}