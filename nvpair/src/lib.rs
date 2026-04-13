#![allow(non_upper_case_globals)]

//! Idiomatic Rust representation of illumos nvlists.
//!
//! This crate converts raw `nvlist_t` pointers (from libnvpair) into
//! pure Rust types with no raw pointers or FFI in the public API.
//!
//! # Example
//!
//! ```ignore
//! use nvpair::{NvList, NvValue};
//!
//! // Given a raw nvlist pointer from some illumos API:
//! let nvl: NvList = unsafe { nvpair::nvlist_to_rust(raw_ptr) };
//!
//! if let Some(NvValue::String(s)) = nvl.lookup("name") {
//!     println!("name = {s}");
//! }
//! ```

use std::ffi::CStr;
use std::os::raw::c_char;

use nvpair_sys::{
    data_type_t, data_type_t_DATA_TYPE_BOOLEAN,
    data_type_t_DATA_TYPE_BOOLEAN_ARRAY, data_type_t_DATA_TYPE_BOOLEAN_VALUE,
    data_type_t_DATA_TYPE_BYTE, data_type_t_DATA_TYPE_BYTE_ARRAY,
    data_type_t_DATA_TYPE_DOUBLE, data_type_t_DATA_TYPE_HRTIME,
    data_type_t_DATA_TYPE_INT16, data_type_t_DATA_TYPE_INT16_ARRAY,
    data_type_t_DATA_TYPE_INT32, data_type_t_DATA_TYPE_INT32_ARRAY,
    data_type_t_DATA_TYPE_INT64, data_type_t_DATA_TYPE_INT64_ARRAY,
    data_type_t_DATA_TYPE_INT8, data_type_t_DATA_TYPE_INT8_ARRAY,
    data_type_t_DATA_TYPE_NVLIST, data_type_t_DATA_TYPE_NVLIST_ARRAY,
    data_type_t_DATA_TYPE_STRING, data_type_t_DATA_TYPE_STRING_ARRAY,
    data_type_t_DATA_TYPE_UINT16, data_type_t_DATA_TYPE_UINT16_ARRAY,
    data_type_t_DATA_TYPE_UINT32, data_type_t_DATA_TYPE_UINT32_ARRAY,
    data_type_t_DATA_TYPE_UINT64, data_type_t_DATA_TYPE_UINT64_ARRAY,
    data_type_t_DATA_TYPE_UINT8, data_type_t_DATA_TYPE_UINT8_ARRAY,
    nvlist_next_nvpair, nvlist_t, nvpair_name, nvpair_t, nvpair_type,
    nvpair_value_boolean_array, nvpair_value_boolean_value,
    nvpair_value_byte, nvpair_value_byte_array, nvpair_value_double,
    nvpair_value_hrtime, nvpair_value_int16, nvpair_value_int16_array,
    nvpair_value_int32, nvpair_value_int32_array, nvpair_value_int64,
    nvpair_value_int64_array, nvpair_value_int8, nvpair_value_int8_array,
    nvpair_value_nvlist, nvpair_value_nvlist_array, nvpair_value_string,
    nvpair_value_string_array, nvpair_value_uint16,
    nvpair_value_uint16_array, nvpair_value_uint32,
    nvpair_value_uint32_array, nvpair_value_uint64,
    nvpair_value_uint64_array, nvpair_value_uint8,
    nvpair_value_uint8_array, uint_t,
};

/// An ordered list of name-value pairs, converted from an illumos nvlist.
#[derive(Debug, Clone, PartialEq)]
pub struct NvList {
    pub pairs: Vec<(String, NvValue)>,
}

impl NvList {
    /// Look up a value by name.
    pub fn lookup(&self, name: &str) -> Option<&NvValue> {
        self.pairs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
}

/// A value from an nvlist pair.
#[derive(Debug, Clone, PartialEq)]
pub enum NvValue {
    /// A valueless boolean (presence indicates true).
    Boolean,
    /// A boolean with an explicit value.
    BooleanValue(bool),
    Byte(u8),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
    String(String),
    /// High-resolution time in nanoseconds.
    Hrtime(i64),
    NvList(NvList),
    BooleanArray(Vec<bool>),
    ByteArray(Vec<u8>),
    Int8Array(Vec<i8>),
    UInt8Array(Vec<u8>),
    Int16Array(Vec<i16>),
    UInt16Array(Vec<u16>),
    Int32Array(Vec<i32>),
    UInt32Array(Vec<u32>),
    Int64Array(Vec<i64>),
    UInt64Array(Vec<u64>),
    StringArray(Vec<String>),
    NvListArray(Vec<NvList>),
    /// A type not recognized by this crate.
    Unknown { type_code: data_type_t },
}

/// Convert a raw `nvlist_t *` into an owned [`NvList`].
///
/// # Safety
///
/// `nvl` must be a valid, non-null pointer to an nvlist. The nvlist is
/// borrowed - the caller retains ownership and is responsible for
/// freeing it.
pub unsafe fn nvlist_to_rust(nvl: *mut nvlist_t) -> NvList {
    let mut pairs = Vec::new();
    let mut nvp: *mut nvpair_t = std::ptr::null_mut();

    loop {
        nvp = unsafe { nvlist_next_nvpair(nvl, nvp) };
        if nvp.is_null() {
            break;
        }

        let name = unsafe {
            CStr::from_ptr(nvpair_name(nvp))
                .to_string_lossy()
                .into_owned()
        };
        let dtype = unsafe { nvpair_type(nvp) };
        let value = unsafe { read_pair_value(nvp, dtype) };
        pairs.push((name, value));
    }

    NvList { pairs }
}

unsafe fn read_pair_value(
    nvp: *mut nvpair_t,
    dtype: data_type_t,
) -> NvValue {
    match dtype {
        data_type_t_DATA_TYPE_BOOLEAN => NvValue::Boolean,
        data_type_t_DATA_TYPE_BOOLEAN_VALUE => {
            let mut v: nvpair_sys::boolean_t = 0;
            nvpair_value_boolean_value(nvp, &mut v);
            NvValue::BooleanValue(v != 0)
        }
        data_type_t_DATA_TYPE_BYTE => {
            let mut v: nvpair_sys::uchar_t = 0;
            nvpair_value_byte(nvp, &mut v);
            NvValue::Byte(v)
        }
        data_type_t_DATA_TYPE_INT8 => {
            let mut v: i8 = 0;
            nvpair_value_int8(nvp, &mut v);
            NvValue::Int8(v)
        }
        data_type_t_DATA_TYPE_UINT8 => {
            let mut v: u8 = 0;
            nvpair_value_uint8(nvp, &mut v);
            NvValue::UInt8(v)
        }
        data_type_t_DATA_TYPE_INT16 => {
            let mut v: i16 = 0;
            nvpair_value_int16(nvp, &mut v);
            NvValue::Int16(v)
        }
        data_type_t_DATA_TYPE_UINT16 => {
            let mut v: u16 = 0;
            nvpair_value_uint16(nvp, &mut v);
            NvValue::UInt16(v)
        }
        data_type_t_DATA_TYPE_INT32 => {
            let mut v: i32 = 0;
            nvpair_value_int32(nvp, &mut v);
            NvValue::Int32(v)
        }
        data_type_t_DATA_TYPE_UINT32 => {
            let mut v: u32 = 0;
            nvpair_value_uint32(nvp, &mut v);
            NvValue::UInt32(v)
        }
        data_type_t_DATA_TYPE_INT64 => {
            let mut v: i64 = 0;
            nvpair_value_int64(nvp, &mut v);
            NvValue::Int64(v)
        }
        data_type_t_DATA_TYPE_UINT64 => {
            let mut v: u64 = 0;
            nvpair_value_uint64(nvp, &mut v);
            NvValue::UInt64(v)
        }
        data_type_t_DATA_TYPE_DOUBLE => {
            let mut v: f64 = 0.0;
            nvpair_value_double(nvp, &mut v);
            NvValue::Double(v)
        }
        data_type_t_DATA_TYPE_STRING => {
            let mut p: *mut c_char = std::ptr::null_mut();
            nvpair_value_string(nvp, &mut p);
            let s = CStr::from_ptr(p).to_string_lossy().into_owned();
            NvValue::String(s)
        }
        data_type_t_DATA_TYPE_HRTIME => {
            let mut v: nvpair_sys::hrtime_t = 0;
            nvpair_value_hrtime(nvp, &mut v);
            NvValue::Hrtime(v)
        }
        data_type_t_DATA_TYPE_NVLIST => {
            let mut p: *mut nvlist_t = std::ptr::null_mut();
            nvpair_value_nvlist(nvp, &mut p);
            NvValue::NvList(nvlist_to_rust(p))
        }
        data_type_t_DATA_TYPE_BOOLEAN_ARRAY => {
            let mut p: *mut nvpair_sys::boolean_t = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_boolean_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::BooleanArray(slice.iter().map(|&v| v != 0).collect())
        }
        data_type_t_DATA_TYPE_BYTE_ARRAY => {
            let mut p: *mut nvpair_sys::uchar_t = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_byte_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::ByteArray(slice.to_vec())
        }
        data_type_t_DATA_TYPE_INT8_ARRAY => {
            let mut p: *mut i8 = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_int8_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::Int8Array(slice.to_vec())
        }
        data_type_t_DATA_TYPE_UINT8_ARRAY => {
            let mut p: *mut u8 = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_uint8_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::UInt8Array(slice.to_vec())
        }
        data_type_t_DATA_TYPE_INT16_ARRAY => {
            let mut p: *mut i16 = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_int16_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::Int16Array(slice.to_vec())
        }
        data_type_t_DATA_TYPE_UINT16_ARRAY => {
            let mut p: *mut u16 = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_uint16_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::UInt16Array(slice.to_vec())
        }
        data_type_t_DATA_TYPE_INT32_ARRAY => {
            let mut p: *mut i32 = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_int32_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::Int32Array(slice.to_vec())
        }
        data_type_t_DATA_TYPE_UINT32_ARRAY => {
            let mut p: *mut u32 = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_uint32_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::UInt32Array(slice.to_vec())
        }
        data_type_t_DATA_TYPE_INT64_ARRAY => {
            let mut p: *mut i64 = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_int64_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::Int64Array(slice.to_vec())
        }
        data_type_t_DATA_TYPE_UINT64_ARRAY => {
            let mut p: *mut u64 = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_uint64_array(nvp, &mut p, &mut n);
            let slice = std::slice::from_raw_parts(p, n as usize);
            NvValue::UInt64Array(slice.to_vec())
        }
        data_type_t_DATA_TYPE_STRING_ARRAY => {
            let mut p: *mut *mut c_char = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_string_array(nvp, &mut p, &mut n);
            let ptrs = std::slice::from_raw_parts(p, n as usize);
            NvValue::StringArray(
                ptrs.iter()
                    .map(|&s| CStr::from_ptr(s).to_string_lossy().into_owned())
                    .collect(),
            )
        }
        data_type_t_DATA_TYPE_NVLIST_ARRAY => {
            let mut p: *mut *mut nvlist_t = std::ptr::null_mut();
            let mut n: uint_t = 0;
            nvpair_value_nvlist_array(nvp, &mut p, &mut n);
            let ptrs = std::slice::from_raw_parts(p, n as usize);
            NvValue::NvListArray(
                ptrs.iter().map(|&nvl| nvlist_to_rust(nvl)).collect(),
            )
        }
        other => NvValue::Unknown { type_code: other },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use nvpair_sys::{
        boolean_t, nvlist_add_boolean, nvlist_add_boolean_array,
        nvlist_add_boolean_value, nvlist_add_byte, nvlist_add_byte_array,
        nvlist_add_double, nvlist_add_hrtime, nvlist_add_int8,
        nvlist_add_int8_array, nvlist_add_int16, nvlist_add_int16_array,
        nvlist_add_int32, nvlist_add_int32_array, nvlist_add_int64,
        nvlist_add_int64_array, nvlist_add_nvlist, nvlist_add_nvlist_array,
        nvlist_add_string, nvlist_add_string_array, nvlist_add_uint8,
        nvlist_add_uint8_array, nvlist_add_uint16, nvlist_add_uint16_array,
        nvlist_add_uint32, nvlist_add_uint32_array, nvlist_add_uint64,
        nvlist_add_uint64_array, nvlist_alloc, nvlist_free, NV_UNIQUE_NAME,
    };

    /// RAII wrapper around a C nvlist for test construction.
    struct NvListBuilder {
        ptr: *mut nvlist_t,
    }

    impl NvListBuilder {
        fn new() -> Self {
            let mut ptr: *mut nvlist_t = std::ptr::null_mut();
            let rc = unsafe { nvlist_alloc(&mut ptr, NV_UNIQUE_NAME, 0) };
            assert_eq!(rc, 0, "nvlist_alloc failed");
            assert!(!ptr.is_null());
            NvListBuilder { ptr }
        }

        fn to_rust(&self) -> NvList {
            unsafe { nvlist_to_rust(self.ptr) }
        }

        fn add_boolean(&self, name: &str) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe { nvlist_add_boolean(self.ptr, cname.as_ptr()) };
            assert_eq!(rc, 0, "nvlist_add_boolean failed for {name}");
            self
        }

        fn add_boolean_value(&self, name: &str, val: bool) -> &Self {
            let cname = CString::new(name).unwrap();
            let cval: boolean_t = if val { 1 } else { 0 };
            let rc = unsafe {
                nvlist_add_boolean_value(self.ptr, cname.as_ptr(), cval)
            };
            assert_eq!(rc, 0, "nvlist_add_boolean_value failed for {name}");
            self
        }

        fn add_byte(&self, name: &str, val: u8) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_byte(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_byte failed for {name}");
            self
        }

        fn add_int8(&self, name: &str, val: i8) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_int8(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_int8 failed for {name}");
            self
        }

        fn add_uint8(&self, name: &str, val: u8) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_uint8(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_uint8 failed for {name}");
            self
        }

        fn add_int16(&self, name: &str, val: i16) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_int16(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_int16 failed for {name}");
            self
        }

        fn add_uint16(&self, name: &str, val: u16) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_uint16(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_uint16 failed for {name}");
            self
        }

        fn add_int32(&self, name: &str, val: i32) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_int32(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_int32 failed for {name}");
            self
        }

        fn add_uint32(&self, name: &str, val: u32) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_uint32(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_uint32 failed for {name}");
            self
        }

        fn add_int64(&self, name: &str, val: i64) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_int64(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_int64 failed for {name}");
            self
        }

        fn add_uint64(&self, name: &str, val: u64) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_uint64(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_uint64 failed for {name}");
            self
        }

        fn add_double(&self, name: &str, val: f64) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_double(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_double failed for {name}");
            self
        }

        fn add_string(&self, name: &str, val: &str) -> &Self {
            let cname = CString::new(name).unwrap();
            let cval = CString::new(val).unwrap();
            let rc = unsafe {
                nvlist_add_string(self.ptr, cname.as_ptr(), cval.as_ptr())
            };
            assert_eq!(rc, 0, "nvlist_add_string failed for {name}");
            self
        }

        fn add_hrtime(&self, name: &str, val: i64) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_hrtime(self.ptr, cname.as_ptr(), val)
            };
            assert_eq!(rc, 0, "nvlist_add_hrtime failed for {name}");
            self
        }

        fn add_nvlist(&self, name: &str, child: &NvListBuilder) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_nvlist(self.ptr, cname.as_ptr(), child.ptr)
            };
            assert_eq!(rc, 0, "nvlist_add_nvlist failed for {name}");
            self
        }

        fn add_boolean_array(&self, name: &str, vals: &[bool]) -> &Self {
            let cname = CString::new(name).unwrap();
            let cvals: Vec<boolean_t> =
                vals.iter().map(|&v| if v { 1 } else { 0 }).collect();
            let rc = unsafe {
                nvlist_add_boolean_array(
                    self.ptr,
                    cname.as_ptr(),
                    cvals.as_ptr() as *mut boolean_t,
                    cvals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_boolean_array failed for {name}");
            self
        }

        fn add_byte_array(&self, name: &str, vals: &[u8]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_byte_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut u8,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_byte_array failed for {name}");
            self
        }

        fn add_int8_array(&self, name: &str, vals: &[i8]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_int8_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut i8,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_int8_array failed for {name}");
            self
        }

        fn add_uint8_array(&self, name: &str, vals: &[u8]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_uint8_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut u8,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_uint8_array failed for {name}");
            self
        }

        fn add_int16_array(&self, name: &str, vals: &[i16]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_int16_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut i16,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_int16_array failed for {name}");
            self
        }

        fn add_uint16_array(&self, name: &str, vals: &[u16]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_uint16_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut u16,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_uint16_array failed for {name}");
            self
        }

        fn add_int32_array(&self, name: &str, vals: &[i32]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_int32_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut i32,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_int32_array failed for {name}");
            self
        }

        fn add_uint32_array(&self, name: &str, vals: &[u32]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_uint32_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut u32,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_uint32_array failed for {name}");
            self
        }

        fn add_int64_array(&self, name: &str, vals: &[i64]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_int64_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut i64,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_int64_array failed for {name}");
            self
        }

        fn add_uint64_array(&self, name: &str, vals: &[u64]) -> &Self {
            let cname = CString::new(name).unwrap();
            let rc = unsafe {
                nvlist_add_uint64_array(
                    self.ptr,
                    cname.as_ptr(),
                    vals.as_ptr() as *mut u64,
                    vals.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_uint64_array failed for {name}");
            self
        }

        fn add_string_array(&self, name: &str, vals: &[&str]) -> &Self {
            let cname = CString::new(name).unwrap();
            let cvals: Vec<CString> =
                vals.iter().map(|s| CString::new(*s).unwrap()).collect();
            let ptrs: Vec<*mut c_char> =
                cvals.iter().map(|c| c.as_ptr() as *mut c_char).collect();
            let rc = unsafe {
                nvlist_add_string_array(
                    self.ptr,
                    cname.as_ptr(),
                    ptrs.as_ptr(),
                    ptrs.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_string_array failed for {name}");
            self
        }

        fn add_nvlist_array(
            &self,
            name: &str,
            children: &[&NvListBuilder],
        ) -> &Self {
            let cname = CString::new(name).unwrap();
            let mut ptrs: Vec<*mut nvlist_t> =
                children.iter().map(|c| c.ptr).collect();
            let rc = unsafe {
                nvlist_add_nvlist_array(
                    self.ptr,
                    cname.as_ptr(),
                    ptrs.as_mut_ptr(),
                    ptrs.len() as uint_t,
                )
            };
            assert_eq!(rc, 0, "nvlist_add_nvlist_array failed for {name}");
            self
        }
    }

    impl Drop for NvListBuilder {
        fn drop(&mut self) {
            unsafe { nvlist_free(self.ptr) }
        }
    }

    // ---- Scalar type tests ----

    #[test]
    fn test_boolean() {
        let nvl = NvListBuilder::new();
        nvl.add_boolean("flag");
        let result = nvl.to_rust();
        assert_eq!(result.lookup("flag"), Some(&NvValue::Boolean));
    }

    #[test]
    fn test_boolean_value_true() {
        let nvl = NvListBuilder::new();
        nvl.add_boolean_value("enabled", true);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("enabled"),
            Some(&NvValue::BooleanValue(true))
        );
    }

    #[test]
    fn test_boolean_value_false() {
        let nvl = NvListBuilder::new();
        nvl.add_boolean_value("enabled", false);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("enabled"),
            Some(&NvValue::BooleanValue(false))
        );
    }

    #[test]
    fn test_byte() {
        let nvl = NvListBuilder::new();
        nvl.add_byte("b", 0xAB);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("b"), Some(&NvValue::Byte(0xAB)));
    }

    #[test]
    fn test_int8() {
        let nvl = NvListBuilder::new();
        nvl.add_int8("v", i8::MIN);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("v"), Some(&NvValue::Int8(i8::MIN)));
    }

    #[test]
    fn test_uint8() {
        let nvl = NvListBuilder::new();
        nvl.add_uint8("v", u8::MAX);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("v"), Some(&NvValue::UInt8(u8::MAX)));
    }

    #[test]
    fn test_int16() {
        let nvl = NvListBuilder::new();
        nvl.add_int16("v", i16::MIN);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("v"), Some(&NvValue::Int16(i16::MIN)));
    }

    #[test]
    fn test_uint16() {
        let nvl = NvListBuilder::new();
        nvl.add_uint16("v", u16::MAX);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("v"), Some(&NvValue::UInt16(u16::MAX)));
    }

    #[test]
    fn test_int32() {
        let nvl = NvListBuilder::new();
        nvl.add_int32("v", i32::MIN);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("v"), Some(&NvValue::Int32(i32::MIN)));
    }

    #[test]
    fn test_uint32() {
        let nvl = NvListBuilder::new();
        nvl.add_uint32("v", u32::MAX);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("v"), Some(&NvValue::UInt32(u32::MAX)));
    }

    #[test]
    fn test_int64() {
        let nvl = NvListBuilder::new();
        nvl.add_int64("v", i64::MIN);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("v"), Some(&NvValue::Int64(i64::MIN)));
    }

    #[test]
    fn test_uint64() {
        let nvl = NvListBuilder::new();
        nvl.add_uint64("v", u64::MAX);
        let result = nvl.to_rust();
        assert_eq!(result.lookup("v"), Some(&NvValue::UInt64(u64::MAX)));
    }

    #[test]
    fn test_double() {
        let nvl = NvListBuilder::new();
        nvl.add_double("pi", std::f64::consts::PI);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("pi"),
            Some(&NvValue::Double(std::f64::consts::PI))
        );
    }

    #[test]
    fn test_string() {
        let nvl = NvListBuilder::new();
        nvl.add_string("greeting", "hello world");
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("greeting"),
            Some(&NvValue::String("hello world".into()))
        );
    }

    #[test]
    fn test_hrtime() {
        let nvl = NvListBuilder::new();
        nvl.add_hrtime("ts", 123_456_789);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("ts"),
            Some(&NvValue::Hrtime(123_456_789))
        );
    }

    // ---- Array type tests ----

    #[test]
    fn test_boolean_array() {
        let nvl = NvListBuilder::new();
        nvl.add_boolean_array("flags", &[true, false, true]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("flags"),
            Some(&NvValue::BooleanArray(vec![true, false, true]))
        );
    }

    #[test]
    fn test_byte_array() {
        let nvl = NvListBuilder::new();
        nvl.add_byte_array("data", &[1, 2, 3]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("data"),
            Some(&NvValue::ByteArray(vec![1, 2, 3]))
        );
    }

    #[test]
    fn test_int8_array() {
        let nvl = NvListBuilder::new();
        nvl.add_int8_array("vals", &[i8::MIN, 0, i8::MAX]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("vals"),
            Some(&NvValue::Int8Array(vec![i8::MIN, 0, i8::MAX]))
        );
    }

    #[test]
    fn test_uint8_array() {
        let nvl = NvListBuilder::new();
        nvl.add_uint8_array("vals", &[0, 128, u8::MAX]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("vals"),
            Some(&NvValue::UInt8Array(vec![0, 128, u8::MAX]))
        );
    }

    #[test]
    fn test_int16_array() {
        let nvl = NvListBuilder::new();
        nvl.add_int16_array("vals", &[i16::MIN, 0, i16::MAX]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("vals"),
            Some(&NvValue::Int16Array(vec![i16::MIN, 0, i16::MAX]))
        );
    }

    #[test]
    fn test_uint16_array() {
        let nvl = NvListBuilder::new();
        nvl.add_uint16_array("vals", &[0, u16::MAX]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("vals"),
            Some(&NvValue::UInt16Array(vec![0, u16::MAX]))
        );
    }

    #[test]
    fn test_int32_array() {
        let nvl = NvListBuilder::new();
        nvl.add_int32_array("vals", &[i32::MIN, 0, i32::MAX]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("vals"),
            Some(&NvValue::Int32Array(vec![i32::MIN, 0, i32::MAX]))
        );
    }

    #[test]
    fn test_uint32_array() {
        let nvl = NvListBuilder::new();
        nvl.add_uint32_array("vals", &[0, u32::MAX]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("vals"),
            Some(&NvValue::UInt32Array(vec![0, u32::MAX]))
        );
    }

    #[test]
    fn test_int64_array() {
        let nvl = NvListBuilder::new();
        nvl.add_int64_array("vals", &[i64::MIN, 0, i64::MAX]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("vals"),
            Some(&NvValue::Int64Array(vec![i64::MIN, 0, i64::MAX]))
        );
    }

    #[test]
    fn test_uint64_array() {
        let nvl = NvListBuilder::new();
        nvl.add_uint64_array("vals", &[0, u64::MAX]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("vals"),
            Some(&NvValue::UInt64Array(vec![0, u64::MAX]))
        );
    }

    #[test]
    fn test_string_array() {
        let nvl = NvListBuilder::new();
        nvl.add_string_array("names", &["alpha", "beta", "gamma"]);
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("names"),
            Some(&NvValue::StringArray(vec![
                "alpha".into(),
                "beta".into(),
                "gamma".into(),
            ]))
        );
    }

    #[test]
    fn test_nvlist_array() {
        let child1 = NvListBuilder::new();
        child1.add_string("name", "first");
        let child2 = NvListBuilder::new();
        child2.add_string("name", "second");

        let nvl = NvListBuilder::new();
        nvl.add_nvlist_array("items", &[&child1, &child2]);

        let result = nvl.to_rust();
        let items = result.lookup("items").unwrap();
        if let NvValue::NvListArray(arr) = items {
            assert_eq!(arr.len(), 2);
            assert_eq!(
                arr[0].lookup("name"),
                Some(&NvValue::String("first".into()))
            );
            assert_eq!(
                arr[1].lookup("name"),
                Some(&NvValue::String("second".into()))
            );
        } else {
            panic!("expected NvListArray, got {items:?}");
        }
    }

    // ---- Structural / edge case tests ----

    #[test]
    fn test_empty_nvlist() {
        let nvl = NvListBuilder::new();
        let result = nvl.to_rust();
        assert!(result.pairs.is_empty());
    }

    #[test]
    fn test_multiple_pairs_in_order() {
        let nvl = NvListBuilder::new();
        nvl.add_string("first", "a");
        nvl.add_int32("second", 42);
        nvl.add_boolean("third");
        nvl.add_uint64("fourth", 999);
        nvl.add_double("fifth", 2.5);

        let result = nvl.to_rust();
        assert_eq!(result.pairs.len(), 5);
        assert_eq!(result.pairs[0].0, "first");
        assert_eq!(result.pairs[1].0, "second");
        assert_eq!(result.pairs[2].0, "third");
        assert_eq!(result.pairs[3].0, "fourth");
        assert_eq!(result.pairs[4].0, "fifth");

        assert_eq!(result.pairs[0].1, NvValue::String("a".into()));
        assert_eq!(result.pairs[1].1, NvValue::Int32(42));
        assert_eq!(result.pairs[2].1, NvValue::Boolean);
        assert_eq!(result.pairs[3].1, NvValue::UInt64(999));
        assert_eq!(result.pairs[4].1, NvValue::Double(2.5));
    }

    #[test]
    fn test_nested_nvlist() {
        let inner = NvListBuilder::new();
        inner.add_string("key", "value");

        let outer = NvListBuilder::new();
        outer.add_nvlist("child", &inner);

        let result = outer.to_rust();
        if let Some(NvValue::NvList(child)) = result.lookup("child") {
            assert_eq!(
                child.lookup("key"),
                Some(&NvValue::String("value".into()))
            );
        } else {
            panic!("expected nested NvList");
        }
    }

    #[test]
    fn test_deeply_nested_nvlist() {
        let innermost = NvListBuilder::new();
        innermost.add_string("depth", "three");

        let middle = NvListBuilder::new();
        middle.add_nvlist("inner", &innermost);

        let outer = NvListBuilder::new();
        outer.add_nvlist("middle", &middle);

        let result = outer.to_rust();
        if let Some(NvValue::NvList(mid)) = result.lookup("middle") {
            if let Some(NvValue::NvList(inn)) = mid.lookup("inner") {
                assert_eq!(
                    inn.lookup("depth"),
                    Some(&NvValue::String("three".into()))
                );
            } else {
                panic!("expected inner NvList");
            }
        } else {
            panic!("expected middle NvList");
        }
    }

    #[test]
    fn test_empty_string() {
        let nvl = NvListBuilder::new();
        nvl.add_string("empty", "");
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("empty"),
            Some(&NvValue::String(String::new()))
        );
    }

    #[test]
    fn test_lookup_missing_key() {
        let nvl = NvListBuilder::new();
        nvl.add_string("exists", "yes");
        let result = nvl.to_rust();
        assert_eq!(
            result.lookup("exists"),
            Some(&NvValue::String("yes".into()))
        );
        assert_eq!(result.lookup("missing"), None);
    }
}
