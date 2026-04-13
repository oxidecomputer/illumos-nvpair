# illumos-nvpair

Idiomatic Rust wrapper around illumos
[libnvpair](https://illumos.org/man/3LIB/libnvpair). Converts raw
`nvlist_t` pointers into pure Rust types - no raw pointers or FFI in the
public API.

## API

- **`NvList`** - an ordered list of name-value pairs, stored as
  `Vec<(String, NvValue)>`. Provides a `lookup(name)` method for
  retrieving values by key.

- **`NvValue`** - an enum covering every nvpair data type: scalars
  (`Boolean`, `Byte`, `Int8`–`UInt64`, `Double`, `String`, `Hrtime`),
  arrays of each, nested `NvList`, and an `Unknown` fallback for
  unrecognized type codes.

- **`nvlist_to_rust(nvl: *mut nvlist_t) -> NvList`** - the single unsafe
  entry point. It borrows the C nvlist pointer, walks every pair via
  `nvlist_next_nvpair`, deep-copies all values into owned Rust types, and
  returns the result. The caller retains ownership of the original
  `nvlist_t` and is responsible for freeing it.

## Design: borrow and copy

The wrapper deep-copies rather than wrapping the `nvlist_t` pointer with
a Rust lifetime or ownership model. This is deliberate: consumers like
[fmd-adm](https://github.com/oxidecomputer/fmd-adm) receive nvlists from
C callbacks where the library owns the memory and the pointer is only
valid for the duration of the callback. Deep-copying into owned Rust
values avoids lifetime entanglement with C and lets the resulting
`NvList` be freely moved, cloned, and stored.
