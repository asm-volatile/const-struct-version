#![doc = include_str!("../README.md")]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::uninlined_format_args, clippy::needless_doctest_main)]

use std::any::type_name;

use sha1::Digest;

#[doc(hidden)]
pub mod __private {
    pub use sha1;

    pub fn execute_if_serde_enabled(hasher: &mut sha1::Sha1, f: impl FnOnce(&mut sha1::Sha1)) {
        if cfg!(feature = "serde-attributes") {
            f(hasher);
        }
    }
}

pub trait StructVersion {
    fn version() -> String;
    fn version_cached() -> &'static str {
        ""
    }
}

#[cfg(feature = "derive")]
pub use const_struct_version_derive::StructVersion;

macro_rules! impl_struct_version {
    ($($t:ty),*) => {
        $(
            impl StructVersion for $t {
                fn version() -> String {
                    let mut hasher = sha1::Sha1::new();
                    hasher.update(type_name::<$t>());
                    format!("{:x}", hasher.finalize())
                }
            }
        )*
    };
}

impl_struct_version!(
    (),
    bool,
    char,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    u128,
    i128,
    f32,
    f64,
    String,
    std::time::Duration,
    std::time::Instant,
    core::net::Ipv4Addr,
    core::net::Ipv6Addr
);

#[cfg_attr(test, mutants::skip)]
impl<T: StructVersion> StructVersion for Vec<T> {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Vec");
        hasher.update(type_name::<T>());
        format!("{:x}", hasher.finalize())
    }
}
impl<T: StructVersion, const N: usize> StructVersion for [T; N] {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Array");
        hasher.update(type_name::<T>());
        hasher.update(N.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }
}
impl<T: StructVersion> StructVersion for Option<T> {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Option");
        hasher.update(type_name::<T>());
        format!("{:x}", hasher.finalize())
    }
}
impl<T: StructVersion, E: StructVersion> StructVersion for Result<T, E> {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Result");
        hasher.update(type_name::<T>());
        hasher.update(type_name::<E>());
        format!("{:x}", hasher.finalize())
    }
}
impl<T: StructVersion> StructVersion for Box<T> {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Box");
        hasher.update(type_name::<T>());
        format!("{:x}", hasher.finalize())
    }
}
impl<T: StructVersion> StructVersion for std::rc::Rc<T> {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Rc");
        hasher.update(type_name::<T>());
        format!("{:x}", hasher.finalize())
    }
}
impl<T: StructVersion> StructVersion for std::sync::Arc<T> {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Arc");
        hasher.update(type_name::<T>());
        format!("{:x}", hasher.finalize())
    }
}

impl<K, V, S> StructVersion for std::collections::HashMap<K, V, S>
where
    K: StructVersion,
    V: StructVersion,
    S: std::hash::BuildHasher,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("HashMap");
        hasher.update(type_name::<K>());
        hasher.update(type_name::<V>());
        format!("{:x}", hasher.finalize())
    }
}

impl<K, V> StructVersion for std::collections::BTreeMap<K, V>
where
    K: StructVersion,
    V: StructVersion,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("BTreeMap");
        hasher.update(type_name::<K>());
        hasher.update(type_name::<V>());
        format!("{:x}", hasher.finalize())
    }
}

impl<K, S> StructVersion for std::collections::HashSet<K, S>
where
    K: StructVersion,
    S: std::hash::BuildHasher,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("HashSet");
        hasher.update(type_name::<K>());
        format!("{:x}", hasher.finalize())
    }
}

impl<K> StructVersion for std::collections::BTreeSet<K>
where
    K: StructVersion,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("BTreeSet");
        hasher.update(type_name::<K>());
        format!("{:x}", hasher.finalize())
    }
}

impl<T> StructVersion for std::collections::LinkedList<T>
where
    T: StructVersion,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("LinkedList");
        hasher.update(type_name::<T>());
        format!("{:x}", hasher.finalize())
    }
}

impl<T> StructVersion for std::collections::VecDeque<T>
where
    T: StructVersion,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("VecDeque");
        hasher.update(type_name::<T>());
        format!("{:x}", hasher.finalize())
    }
}

impl<T> StructVersion for std::collections::BinaryHeap<T>
where
    T: StructVersion,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("BinaryHeap");
        hasher.update(type_name::<T>());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(feature = "chrono")]
impl<Tz: chrono::TimeZone> StructVersion for chrono::DateTime<Tz> {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("DateTime");
        hasher.update(type_name::<Tz>());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(feature = "url")]
impl StructVersion for url::Url {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Url");
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(feature = "uuid")]
impl StructVersion for uuid::Uuid {
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("Uuid");
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(feature = "indexmap")]
impl<K, V, S> StructVersion for indexmap::IndexMap<K, V, S>
where
    K: StructVersion,
    V: StructVersion,
    S: std::hash::BuildHasher,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("IndexMap");
        hasher.update(type_name::<K>());
        hasher.update(type_name::<V>());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(feature = "indexmap")]
impl<K> StructVersion for indexmap::IndexSet<K>
where
    K: StructVersion,
{
    fn version() -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update("IndexSet");
        hasher.update(type_name::<K>());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    macro_rules! test {
        ($name:ident, $t:ty) => {
            #[test]
            fn $name() {
                let version = <$t as super::StructVersion>::version();
                insta::assert_snapshot!(version);
            }
        };
    }

    test!(test_bool, bool);
    test!(test_char, char);
    test!(test_u8, u8);
    test!(test_i8, i8);
    test!(test_u16, u16);
    test!(test_i16, i16);
    test!(test_u32, u32);
    test!(test_i32, i32);
    test!(test_u64, u64);
    test!(test_i64, i64);
    test!(test_u128, u128);
    test!(test_i128, i128);
    test!(test_f32, f32);
    test!(test_f64, f64);
    test!(test_string, String);
    test!(test_duration, std::time::Duration);
    test!(test_instant, std::time::Instant);
    test!(test_vec_u32, Vec<u32>);
    test!(test_option_u32, Option<u32>);
    test!(test_box_u32, Box<u32>);
    test!(test_rc_u32, std::rc::Rc<u32>);
    test!(test_arc_u32, std::sync::Arc<u32>);
    test!(test_hash_map_u32_u32, std::collections::HashMap<u32, u32>);
    test!(test_b_tree_map_u32_u32, std::collections::BTreeMap<u32, u32>);
    test!(test_hash_set_u32, std::collections::HashSet<u32>);
    test!(test_b_tree_set_u32, std::collections::BTreeSet<u32>);
    test!(test_linked_list_u32, std::collections::LinkedList<u32>);
    test!(test_vec_deque_u32, std::collections::VecDeque<u32>);
    test!(test_binary_heap_u32, std::collections::BinaryHeap<u32>);
    #[cfg(feature = "chrono")]
    test!(test_date_time_local, chrono::DateTime<chrono::Local>);
    #[cfg(feature = "url")]
    test!(test_url, url::Url);
    #[cfg(feature = "uuid")]
    test!(test_uuid, uuid::Uuid);
    #[cfg(feature = "indexmap")]
    test!(test_index_map_u32_u32, indexmap::IndexMap<u32, u32>);
    #[cfg(feature = "indexmap")]
    test!(test_index_set_u32, indexmap::IndexSet<u32>);
}
