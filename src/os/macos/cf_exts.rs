//! Extensions to the [`core_foundation`] crate.
//!
//! At attempt should be made to upstream an improved version of these back into
//! `core_foundation` at some point.

// TODO: Remove this if upstreaming this code into core-foundation.
#![allow(dead_code)]

use std::{
    ffi::{CStr, CString},
    path::Path,
};

use cf::url::kCFURLPOSIXPathStyle;
use core_foundation::{
    self as cf,
    array::CFArrayRef,
    base::{CFIndex, CFTypeRef},
    bundle::CFBundleRef,
    error::CFErrorRef,
    string::{
        kCFStringEncodingUTF8, CFStringCreateWithCString, CFStringEncoding,
        CFStringGetBytes, CFStringGetCStringPtr, CFStringGetLength, CFStringRef,
    },
    url::{
        CFURLCopyAbsoluteURL, CFURLCopyFileSystemPath, CFURLCopyPath,
        CFURLCreateWithFileSystemPath, CFURLGetFileSystemRepresentation, CFURLGetString,
        CFURLRef,
    },
};

//======================================
// Begin CoreFoundation bindings interlude
//======================================

extern "C" {
    pub fn CFBundleGetIdentifier(bundle: CFBundleRef) -> CFStringRef;

    fn CFBundleGetValueForInfoDictionaryKey(
        bundle: CFBundleRef,
        key: CFStringRef,
    ) -> CFTypeRef;

    pub fn LSCopyApplicationURLsForBundleIdentifier(
        inBundleIdentifier: CFStringRef,
        outError: *mut CFErrorRef,
    ) -> CFArrayRef;

    pub fn CFStringGetMaximumSizeForEncoding(
        length: CFIndex,
        encoding: CFStringEncoding,
    ) -> CFIndex;
}

pub unsafe fn get_cf_string(cf_str: CFStringRef) -> Option<String> {
    // First try to efficiently get a pointer to the C string data.
    // If the `cf_str` is not stored as a C string, this will fail.
    let c_str = CFStringGetCStringPtr(cf_str, kCFStringEncodingUTF8);
    if !c_str.is_null() {
        let id = CStr::from_ptr(c_str);
        // TODO: Instead of returning None here if this isn't valid UTF, continue to
        //       doing the conversion below.
        let id_str = id.to_str().ok()?;

        return Some(id_str.to_owned());
    }

    //----------------------------------------
    // Fall back to copying the C string data.
    // First determine the maximum buffer size we could need.
    //----------------------------------------

    // Number (in terms of UTF-16 code pairs) of Unicode characters in the string.
    let string_char_length: CFIndex = CFStringGetLength(cf_str);

    // Maximum number of bytes necessary to store a unicode string with the specified
    // number of characters in encoded UTF-8 format.
    let buffer_max_cf_size: CFIndex =
        CFStringGetMaximumSizeForEncoding(string_char_length, kCFStringEncodingUTF8);
    let buffer_max_size = usize::try_from(buffer_max_cf_size)
        .expect("string maximum buffer length overflows usize");

    let mut buffer = Vec::with_capacity(buffer_max_size);

    //-------------------------------------------
    // Copy the string contents, encoded as UTF-8
    //-------------------------------------------

    // TODO: Use CFStringGetBytes() here instead, to avoid the extra allocation to convert
    //       from CString -> String. Instead we could use String::from_vec().
    let mut used_buffer_len: CFIndex = 0;
    let converted_char_length = CFStringGetBytes(
        cf_str,
        cf::base::CFRange::init(0, string_char_length),
        kCFStringEncodingUTF8,
        0,           // Don't lossily encode bytes, fail.
        false as u8, // Don't use an "external representation" (containing byte order marks).
        buffer.as_mut_ptr(),
        buffer_max_cf_size,
        &mut used_buffer_len,
    );

    if converted_char_length != string_char_length {
        return None;
    }

    let used_buffer_len: usize = usize::try_from(used_buffer_len)
        .expect("CFStringGetBytes() used buffer length overflows usize");

    // Only this many bytes will have been initialized, so this is the length.
    buffer.set_len(used_buffer_len);

    // TODO: `buffer.shrink_to_fit()`? Perhaps mention this as something the caller can
    //       optionally do if they want to conserve memory.

    // TODO: Panic if this fails?
    String::from_utf8(buffer).ok()
}

/// # Panics
///
/// This function will panic if the underlying call to `CFStringCreateWithCString()`
/// fails.
pub fn cf_string_from_cstr(cstr: &CStr) -> CFStringRef {
    let cf_string: CFStringRef = unsafe {
        // Use the default allocator.
        let allocator = std::ptr::null();

        CFStringCreateWithCString(allocator, cstr.as_ptr(), kCFStringEncodingUTF8)
    };

    if cf_string.is_null() {
        panic!("unable to create CFStringRef from &CStr")
    }

    cf_string
}

pub fn cf_string_from_str(str: &str) -> CFStringRef {
    let cstring = CString::new(str).expect("unable to create CString from &str");
    cf_string_from_cstr(&cstring)
}

//--------------------------------------
// CFBundle
//--------------------------------------

pub unsafe fn bundle_identifier(bundle: CFBundleRef) -> Option<String> {
    let id_cf_str: CFStringRef = CFBundleGetIdentifier(bundle);

    if id_cf_str.is_null() {
        return None;
    }

    get_cf_string(id_cf_str)
}

/// *CoreFoundation API Documentation*:
/// [`CFBundleGetValueForInfoDictionaryKey`](https://developer.apple.com/documentation/corefoundation/1537102-cfbundlegetvalueforinfodictionar?language=objc)
pub unsafe fn bundle_get_value_for_info_dictionary_key(
    bundle: CFBundleRef,
    key: &str,
) -> Option<String> {
    let key_cstring = CString::new(key).expect("");

    let key_cfstring = cf_string_from_cstr(&key_cstring);

    let value: CFTypeRef = CFBundleGetValueForInfoDictionaryKey(bundle, key_cfstring);

    // println!("value type id: {}", cf::base::CFGetTypeID(value));
    // println!(
    //     "string type id: {}",
    //     cf::base::CFGetTypeID(cf_string_from_str("hello") as *const _)
    // );

    // FIXME: Assert that `value`'s dynamic type is actually CFStringRef.
    let value: CFStringRef = value as CFStringRef;

    if !value.is_null() {
        let name: String = match get_cf_string(value) {
            Some(name) => name,
            None => panic!(
                "CFBundleRef info dictionary value for key '{}' was invalid",
                key
            ),
        };
        return Some(name);
    } else {
        None
    }
}

//--------------------------------------
// CFURL
//--------------------------------------

pub unsafe fn url_absolute_url(url: CFURLRef) -> CFURLRef {
    CFURLCopyAbsoluteURL(url)
}

/// *CoreFoundation API Documentation*:
/// [`CFURLCopyPath`](https://developer.apple.com/documentation/corefoundation/1541982-cfurlcopypath?language=objc)
pub unsafe fn url_path(url: CFURLRef) -> String {
    get_cf_string(CFURLCopyPath(url)).expect("CFURLRef path does not exist or is invalid")
}

pub unsafe fn url_file_system_path(url: CFURLRef) -> String {
    get_cf_string(CFURLCopyFileSystemPath(url, kCFURLPOSIXPathStyle))
        .expect("CFURLRef file system path does not exist or is invalid")
}

/// *CoreFoundation API Documentation*:
/// [`CFURLGetFileSystemRepresentation`](https://developer.apple.com/documentation/corefoundation/1541515-cfurlgetfilesystemrepresentation?changes=_4&language=objc)
pub unsafe fn url_get_file_system_representation<T: for<'s> From<&'s str>>(
    url: CFURLRef,
) -> Option<T> {
    const SIZE: usize = 1024;

    let mut buffer: [u8; SIZE] = [0; SIZE];

    let was_successful: bool = CFURLGetFileSystemRepresentation(
        url,
        true as u8,
        buffer.as_mut_ptr(),
        (SIZE - 1) as isize,
    ) > 0;

    if !was_successful {
        return None;
    }

    let cstr = CStr::from_ptr(buffer.as_ptr() as *const i8);

    // TODO: CFURLGetFileSystemRepresentation doesn't state exactly what encoding the
    //       output buffer will have, so it's unclear if this can be expected to work in
    //       all cases.
    let str = cstr
        .to_str()
        .expect("CFURLRef file system representation was not valid UTF-8");

    Some(T::from(str))
}

pub unsafe fn url_get_string(url: CFURLRef) -> CFStringRef {
    CFURLGetString(url)
}

pub fn url_create_with_file_system_path(path: &Path) -> Option<CFURLRef> {
    let path_str: &str = path.to_str()?;

    let path_cfstring: CFStringRef = cf_string_from_str(path_str);

    let url: CFURLRef = unsafe {
        // Use the default allocator.
        let allocator = std::ptr::null();

        // TODO: If `path` is not absolute, this function will resolve it relative to
        //       the current working directory. Is that the behavior we want? Should we
        //       panic if `!path.is_absolute()`?
        CFURLCreateWithFileSystemPath(
            allocator,
            path_cfstring,
            cf::url::kCFURLPOSIXPathStyle,
            path.is_dir() as u8,
        )
    };

    if url.is_null() {
        None
    } else {
        Some(url)
    }
}

//======================================
// End CoreFoundation bindings interlude
//======================================
