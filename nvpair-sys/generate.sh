#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")"

bindgen wrapper.h \
  --allowlist-function 'nvlist_.*' \
  --allowlist-function 'nvpair_.*' \
  --allowlist-var 'NV_.*' \
  --allowlist-var 'data_type_t_.*' \
  --raw-line '#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]' \
  > src/lib.rs
