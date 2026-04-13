# illumos-nvpair

Idiomatic Rust wrapper around illumos
[libnvpair](https://illumos.org/man/3LIB/libnvpair). Converts raw
`nvlist_t` pointers into pure Rust types - no raw pointers or FFI in the
public API.

## API

- **`NvList`** - an ordered list of name-value pairs. Provides
  `lookup(name)` for retrieving values by key, `iter()` for borrowing
  pairs as `(&str, &NvValue)`, and `len()` / `is_empty()`. Implements
  `IntoIterator` for both owned and borrowed iteration.

- **`NvValue`** - an enum covering every nvpair data type: scalars
  (`Boolean`, `Byte`, `Int8`-`UInt64`, `Double`, `String`, `Hrtime`),
  arrays of each, nested `NvList`, and an `Unknown` fallback for
  unrecognized type codes.

- **`NvList::from_raw(nvl: *mut nvlist_t) -> Result<NvList, NvError>`** -
  the single unsafe entry point. It borrows the C nvlist pointer, walks
  every pair via `nvlist_next_nvpair`, deep-copies all values into owned
  Rust types, and returns the result. All C return codes and pointers
  are checked; errors are returned as `NvError` rather than panicking.
  The caller retains ownership of the original `nvlist_t` and is
  responsible for freeing it.

- **`NvError`** - error type with variants for failed C calls
  (`ValueReadFailed`), null pointers (`NullPointer`), and null pair
  names (`NullName`).

## UTF-8 handling

Pair names and string values are converted using lossy UTF-8 conversion:
any bytes that are not valid UTF-8 are replaced with U+FFFD. This means
a `lookup()` call will not match the original name if it contained
non-UTF-8 bytes.

## Design: borrow and copy

The wrapper deep-copies rather than wrapping the `nvlist_t` pointer with
a Rust lifetime or ownership model. This is deliberate: consumers like
[fmd-adm](https://github.com/oxidecomputer/fmd-adm) receive nvlists from
C callbacks where the library owns the memory and the pointer is only
valid for the duration of the callback. Deep-copying into owned Rust
values avoids lifetime entanglement with C and lets the resulting
`NvList` be freely moved, cloned, and stored.
