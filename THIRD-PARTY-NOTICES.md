# Third-party notices

Addons Forever bundles the following third-party work. Full license texts ship in
the source registry for each crate (`cargo` vendoring metadata) and in the crates
themselves; the summary obligations below are met by this file plus the source.

## Notable obligations

- `option-ext` 0.2.0 (MPL-2.0, via `dirs`): file-level copyleft. Source remains
  available in this project's dependency tree; MPL notices retained in the crate.
- Fonts embedded via `epaint_default_fonts` (egui default fonts): Ubuntu-Light
  (UFL-1.1) and Hack, NotoEmoji and others (OFL-1.1). OFL.txt / UFL.txt are
  shipped unmodified with the crate and apply to the font files only.
- `webpki-roots` (CDLA-Permissive-2.0): attribution given here.
- BSD-3-Clause components (e.g. `encoding_rs`): notices retained as above.
- Dual-licensed crates: you may choose MIT or Apache-2.0 for any crate offering both,
  including `self_cell` (Apache-2.0 OR GPL-2.0-only: Apache-2.0 chosen).

## Direct dependencies

| Crate | License |
|---|---|
| `anyhow` 1.0.104 | MIT OR Apache-2.0 |
| `async-trait` 0.1.92 | MIT OR Apache-2.0 |
| `chrono` 0.4.45 | MIT OR Apache-2.0 |
| `dirs` 7.0.0 | MIT OR Apache-2.0 |
| `encoding_rs` 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| `futures` 0.3.34 | MIT OR Apache-2.0 |
| `notify` 8.2.1 | CC0-1.0 |
| `notify-debouncer-full` 0.7.0 | MIT OR Apache-2.0 |
| `rust-ini` 0.21.3 | MIT |
| `reqwest` 0.13.5 | MIT OR Apache-2.0 |
| `rustls` 0.23.46 | Apache-2.0 OR ISC OR MIT |
| `semver` 1.0.28 | MIT OR Apache-2.0 |
| `serde` 1.0.229 | MIT OR Apache-2.0 |
| `serde_json` 1.0.154 | MIT OR Apache-2.0 |
| `thiserror` 2.0.21 | MIT OR Apache-2.0 |
| `tokio` 1.54.1 | MIT |
| `tracing` 0.1.44 | MIT |
| `tracing-subscriber` 0.3.23 | MIT |
| `uuid` 1.29.0 | Apache-2.0 OR MIT |
| `zip` 9.4.0 | MIT |
| `md-5` 0.10.6 | MIT OR Apache-2.0 |
| `sha1` 0.10.7 | MIT OR Apache-2.0 |
| `eframe` 0.37.0 | MIT OR Apache-2.0 |
| `egui` 0.37.0 | MIT OR Apache-2.0 |
| `egui_extras` 0.37.0 | MIT OR Apache-2.0 |
| `interprocess` 2.4.7 | 0BSD OR Apache-2.0 |
| `open` 5.4.5 | MIT |
| `png` 0.18.1 | MIT OR Apache-2.0 |
| `rfd` 0.15.4 | MIT |
| `self_update` 1.3.3 | MIT |
| `tracing-appender` 0.2.5 | MIT |

## Full dependency list

342 crates (runtime graph).

| Crate | Version | License |
|---|---|---|
| `accesskit` | 0.25.2 | MIT OR Apache-2.0 |
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| `ahash` | 0.8.12 | MIT OR Apache-2.0 |
| `aho-corasick` | 1.1.5 | Unlicense OR MIT |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 |
| `arrayvec` | 0.7.8 | MIT OR Apache-2.0 |
| `as-raw-xcb-connection` | 1.0.1 | MIT OR Apache-2.0 |
| `ashpd` | 0.11.1 | MIT |
| `async-broadcast` | 0.7.2 | MIT OR Apache-2.0 |
| `async-channel` | 2.5.0 | Apache-2.0 OR MIT |
| `async-compression` | 0.4.52 | MIT OR Apache-2.0 |
| `async-executor` | 1.14.1 | Apache-2.0 OR MIT |
| `async-fs` | 2.2.0 | Apache-2.0 OR MIT |
| `async-io` | 2.6.0 | Apache-2.0 OR MIT |
| `async-lock` | 3.5.0 | Apache-2.0 OR MIT |
| `async-net` | 2.0.0 | Apache-2.0 OR MIT |
| `async-process` | 2.5.0 | Apache-2.0 OR MIT |
| `async-recursion` | 1.2.0 | MIT OR Apache-2.0 |
| `async-signal` | 0.2.14 | Apache-2.0 OR MIT |
| `async-task` | 4.7.1 | Apache-2.0 OR MIT |
| `async-trait` | 0.1.92 | MIT OR Apache-2.0 |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT |
| `base64` | 0.23.1 | MIT OR Apache-2.0 |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 |
| `block-buffer` | 0.10.4 | MIT OR Apache-2.0 |
| `blocking` | 1.7.0 | Apache-2.0 OR MIT |
| `bumpalo` | 3.20.3 | MIT OR Apache-2.0 |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| `bytemuck_derive` | 1.12.1 | Zlib OR Apache-2.0 OR MIT |
| `byteorder-lite` | 0.1.0 | Unlicense OR MIT |
| `bytes` | 1.12.1 | MIT |
| `calloop` | 0.13.0 | MIT |
| `calloop-wayland-source` | 0.3.0 | MIT |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 |
| `chrono` | 0.4.45 | MIT OR Apache-2.0 |
| `color` | 0.3.3 | Apache-2.0 OR MIT |
| `compression-codecs` | 0.4.47 | MIT OR Apache-2.0 |
| `compression-core` | 0.4.34 | MIT OR Apache-2.0 |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT |
| `const-random` | 0.1.18 | MIT OR Apache-2.0 |
| `const-random-macro` | 0.1.16 | MIT OR Apache-2.0 |
| `core_detect` | 1.0.0 | MIT/Apache-2.0 |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 |
| `crc32fast` | 1.5.4 | MIT OR Apache-2.0 |
| `crossbeam-channel` | 0.5.17 | MIT OR Apache-2.0 |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 |
| `crunchy` | 0.2.4 | MIT |
| `crypto-common` | 0.1.7 | MIT OR Apache-2.0 |
| `cursor-icon` | 1.2.0 | MIT OR Apache-2.0 OR Zlib |
| `deranged` | 0.5.9 | MIT OR Apache-2.0 |
| `digest` | 0.10.7 | MIT OR Apache-2.0 |
| `dirs` | 7.0.0 | MIT OR Apache-2.0 |
| `dirs-sys` | 0.5.0 | MIT OR Apache-2.0 |
| `displaydoc` | 0.2.7 | MIT OR Apache-2.0 |
| `dlib` | 0.5.3 | MIT |
| `dlv-list` | 0.5.2 | MIT OR Apache-2.0 |
| `doctest-file` | 1.1.1 | 0BSD |
| `document-features` | 0.2.12 | MIT OR Apache-2.0 |
| `downcast-rs` | 1.2.1 | MIT/Apache-2.0 |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT |
| `ecolor` | 0.37.0 | MIT OR Apache-2.0 |
| `eframe` | 0.37.0 | MIT OR Apache-2.0 |
| `egui` | 0.37.0 | MIT OR Apache-2.0 |
| `egui-winit` | 0.37.0 | MIT OR Apache-2.0 |
| `egui_extras` | 0.37.0 | MIT OR Apache-2.0 |
| `egui_glow` | 0.37.0 | MIT OR Apache-2.0 |
| `either` | 1.19.0 | MIT OR Apache-2.0 |
| `emath` | 0.37.0 | MIT OR Apache-2.0 |
| `encoding_rs` | 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause |
| `endi` | 1.1.1 | MIT |
| `enum-map` | 3.1.0 | MIT OR Apache-2.0 |
| `enum-map-derive` | 1.0.1 | MIT OR Apache-2.0 |
| `enumflags2` | 0.7.12 | MIT OR Apache-2.0 |
| `enumflags2_derive` | 0.7.12 | MIT OR Apache-2.0 |
| `enumn` | 0.1.14 | MIT OR Apache-2.0 |
| `epaint` | 0.37.0 | MIT OR Apache-2.0 |
| `epaint_default_fonts` | 0.37.0 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 |
| `equivalent` | 1.0.3 | Apache-2.0 OR MIT |
| `errno` | 0.3.14 | MIT OR Apache-2.0 |
| `euclid` | 0.22.14 | MIT OR Apache-2.0 |
| `event-listener` | 5.4.2 | Apache-2.0 OR MIT |
| `event-listener-strategy` | 0.5.4 | Apache-2.0 OR MIT |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 |
| `fearless_simd` | 0.4.1 | Apache-2.0 OR MIT |
| `file-id` | 0.2.3 | MIT OR Apache-2.0 |
| `flate2` | 1.1.10 | MIT OR Apache-2.0 |
| `fnv` | 1.0.7 | Apache-2.0 / MIT |
| `foldhash` | 0.2.0 | Zlib |
| `font-types` | 0.12.7 | MIT OR Apache-2.0 |
| `form_urlencoded` | 1.2.2 | MIT OR Apache-2.0 |
| `futures` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-channel` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-executor` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-io` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-lite` | 2.6.1 | Apache-2.0 OR MIT |
| `futures-macro` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-sink` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 |
| `generic-array` | 0.14.7 | MIT |
| `gethostname` | 1.1.0 | Apache-2.0 |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 |
| `getrandom` | 0.3.4 | MIT OR Apache-2.0 |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 |
| `glow` | 0.17.0 | MIT OR Apache-2.0 OR Zlib |
| `glutin` | 0.32.3 | Apache-2.0 |
| `glutin-winit` | 0.5.0 | MIT |
| `glutin_egl_sys` | 0.7.1 | Apache-2.0 |
| `glutin_glx_sys` | 0.6.1 | Apache-2.0 |
| `guillotiere` | 0.7.0 | MIT/Apache-2.0 |
| `h2` | 0.4.20 | MIT |
| `harfrust` | 0.12.0 | MIT |
| `hashbrown` | 0.14.5 | MIT OR Apache-2.0 |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 |
| `hex` | 0.4.3 | MIT OR Apache-2.0 |
| `http` | 1.5.0 | MIT OR Apache-2.0 |
| `http-body` | 1.1.0 | MIT |
| `http-body-util` | 0.1.5 | MIT |
| `httparse` | 1.10.1 | MIT OR Apache-2.0 |
| `hyper` | 1.12.0 | MIT |
| `hyper-rustls` | 0.27.10 | Apache-2.0 OR ISC OR MIT |
| `hyper-util` | 0.1.21 | MIT |
| `iana-time-zone` | 0.1.65 | MIT OR Apache-2.0 |
| `icu_collections` | 2.4.0 | Unicode-3.0 |
| `icu_locale_core` | 2.4.0 | Unicode-3.0 |
| `icu_normalizer` | 2.4.0 | Unicode-3.0 |
| `icu_normalizer_data` | 2.4.0 | Unicode-3.0 |
| `icu_properties` | 2.4.0 | Unicode-3.0 |
| `icu_properties_data` | 2.4.0 | Unicode-3.0 |
| `icu_provider` | 2.4.0 | Unicode-3.0 |
| `idna` | 1.1.0 | MIT OR Apache-2.0 |
| `idna_adapter` | 1.2.2 | Apache-2.0 OR MIT |
| `image` | 0.25.10 | MIT OR Apache-2.0 |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT |
| `inotify` | 0.11.5 | ISC |
| `inotify-sys` | 0.1.8 | ISC |
| `interprocess` | 2.4.7 | 0BSD OR Apache-2.0 |
| `ipnet` | 2.12.2 | MIT OR Apache-2.0 |
| `is-docker` | 0.2.0 | MIT |
| `is-wsl` | 0.4.0 | MIT |
| `itertools` | 0.15.0 | MIT OR Apache-2.0 |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 |
| `kurbo` | 0.13.1 | Apache-2.0 OR MIT |
| `lazy_static` | 1.5.1 | MIT OR Apache-2.0 |
| `libc` | 0.2.191 | MIT OR Apache-2.0 |
| `libloading` | 0.8.9 | ISC |
| `linebender_resource_handle` | 0.1.1 | Apache-2.0 OR MIT |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `linux-raw-sys` | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `litemap` | 0.8.3 | Unicode-3.0 |
| `litrs` | 1.0.1 | MIT OR Apache-2.0 |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 |
| `log` | 0.4.35 | MIT OR Apache-2.0 |
| `matchers` | 0.2.0 | MIT |
| `md-5` | 0.10.6 | MIT OR Apache-2.0 |
| `memchr` | 2.8.3 | Unlicense OR MIT |
| `memmap2` | 0.9.12 | MIT OR Apache-2.0 |
| `mime` | 0.3.18 | MIT OR Apache-2.0 |
| `mime_guess2` | 2.3.1 | MIT |
| `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| `mio` | 1.2.4 | MIT |
| `moxcms` | 0.8.1 | BSD-3-Clause OR Apache-2.0 |
| `multiversion_no_op` | 1.0.0 | Apache-2.0 OR MIT |
| `nohash-hasher` | 0.2.0 | Apache-2.0 OR MIT |
| `notify` | 8.2.1 | CC0-1.0 |
| `notify-debouncer-full` | 0.7.0 | MIT OR Apache-2.0 |
| `notify-types` | 2.2.0 | MIT OR Apache-2.0 |
| `nu-ansi-term` | 0.50.3 | MIT |
| `num-conv` | 0.2.2 | MIT OR Apache-2.0 |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 |
| `open` | 5.4.5 | MIT |
| `openssl-probe` | 0.2.2 | MIT OR Apache-2.0 |
| `option-ext` | 0.2.0 | MPL-2.0 |
| `ordered-multimap` | 0.7.3 | MIT |
| `ordered-stream` | 0.2.0 | MIT OR Apache-2.0 |
| `parking` | 2.2.1 | Apache-2.0 OR MIT |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 |
| `peniko` | 0.6.1 | Apache-2.0 OR MIT |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT |
| `piper` | 0.2.5 | MIT OR Apache-2.0 |
| `png` | 0.18.1 | MIT OR Apache-2.0 |
| `polling` | 3.11.0 | Apache-2.0 OR MIT |
| `pollster` | 0.4.0 | Apache-2.0/MIT |
| `polycool` | 0.4.0 | MIT OR Apache-2.0 |
| `potential_utf` | 0.1.6 | Unicode-3.0 |
| `powerfmt` | 0.2.1 | MIT OR Apache-2.0 |
| `ppv-lite86` | 0.2.21 | MIT OR Apache-2.0 |
| `proc-macro-crate` | 3.5.0 | MIT OR Apache-2.0 |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 |
| `profiling` | 1.0.18 | MIT OR Apache-2.0 |
| `pxfm` | 0.1.30 | BSD-3-Clause OR Apache-2.0 |
| `quick-xml` | 0.41.0 | MIT |
| `quote` | 1.0.47 | MIT OR Apache-2.0 |
| `rand` | 0.9.5 | MIT OR Apache-2.0 |
| `rand_chacha` | 0.9.0 | MIT OR Apache-2.0 |
| `rand_core` | 0.9.5 | MIT OR Apache-2.0 |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| `read-fonts` | 0.41.0 | MIT OR Apache-2.0 |
| `regex` | 1.13.1 | MIT OR Apache-2.0 |
| `regex-automata` | 0.4.18 | MIT OR Apache-2.0 |
| `regex-syntax` | 0.8.11 | MIT OR Apache-2.0 |
| `reqwest` | 0.13.5 | MIT OR Apache-2.0 |
| `rfd` | 0.15.4 | MIT |
| `ring` | 0.17.14 | Apache-2.0 AND ISC |
| `ron` | 0.12.3 | MIT OR Apache-2.0 |
| `rust-ini` | 0.21.3 | MIT |
| `rustix` | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `rustix` | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `rustls` | 0.23.46 | Apache-2.0 OR ISC OR MIT |
| `rustls-native-certs` | 0.8.5 | Apache-2.0 OR ISC OR MIT |
| `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 |
| `rustls-platform-verifier` | 0.7.1 | MIT OR Apache-2.0 |
| `rustls-webpki` | 0.103.16 | ISC |
| `same-file` | 1.0.6 | Unlicense/MIT |
| `scoped-tls` | 1.0.1 | MIT/Apache-2.0 |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 |
| `self-replace` | 1.5.1 | Apache-2.0 |
| `self_cell` | 1.3.0 | Apache-2.0 OR GPL-2.0-only |
| `self_update` | 1.3.3 | MIT |
| `semver` | 1.0.28 | MIT OR Apache-2.0 |
| `serde` | 1.0.229 | MIT OR Apache-2.0 |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 |
| `serde_repr` | 0.1.21 | MIT OR Apache-2.0 |
| `sha1` | 0.10.7 | MIT OR Apache-2.0 |
| `sharded-slab` | 0.1.7 | MIT |
| `signal-hook-registry` | 1.4.8 | MIT OR Apache-2.0 |
| `simd-adler32` | 0.3.10 | MIT |
| `simdutf8` | 0.1.5 | MIT OR Apache-2.0 |
| `skrifa` | 0.44.0 | MIT OR Apache-2.0 |
| `slab` | 0.4.12 | MIT |
| `smallvec` | 1.16.4 | MIT OR Apache-2.0 |
| `smithay-client-toolkit` | 0.19.2 | MIT |
| `smol_str` | 0.2.2 | MIT OR Apache-2.0 |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 |
| `stable_deref_trait` | 1.2.1 | MIT OR Apache-2.0 |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 |
| `subtle` | 2.6.1 | BSD-3-Clause |
| `symlink` | 0.1.0 | MIT/Apache-2.0 |
| `syn` | 2.0.119 | MIT OR Apache-2.0 |
| `syn` | 3.0.9 | MIT OR Apache-2.0 |
| `sync_wrapper` | 1.0.2 | Apache-2.0 |
| `synstructure` | 0.14.0 | MIT |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 |
| `thread_local` | 1.1.10 | MIT OR Apache-2.0 |
| `time` | 0.3.55 | MIT OR Apache-2.0 |
| `time-core` | 0.1.9 | MIT OR Apache-2.0 |
| `tiny-keccak` | 2.0.2 | CC0-1.0 |
| `tinystr` | 0.8.5 | Unicode-3.0 |
| `tokio` | 1.54.1 | MIT |
| `tokio-macros` | 2.7.2 | MIT |
| `tokio-rustls` | 0.26.6 | MIT OR Apache-2.0 |
| `tokio-util` | 0.7.20 | MIT |
| `toml_datetime` | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 |
| `toml_edit` | 0.25.18+spec-1.1.0 | MIT OR Apache-2.0 |
| `toml_parser` | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 |
| `tower` | 0.5.3 | MIT |
| `tower-http` | 0.6.11 | MIT |
| `tower-layer` | 0.3.3 | MIT |
| `tower-service` | 0.3.3 | MIT |
| `tracing` | 0.1.44 | MIT |
| `tracing-appender` | 0.2.5 | MIT |
| `tracing-attributes` | 0.1.31 | MIT |
| `tracing-core` | 0.1.36 | MIT |
| `tracing-log` | 0.2.0 | MIT |
| `tracing-subscriber` | 0.3.23 | MIT |
| `try-lock` | 0.2.5 | MIT |
| `typed-path` | 0.13.0 | MIT OR Apache-2.0 |
| `typeid` | 1.0.3 | MIT OR Apache-2.0 |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 |
| `unicase` | 2.10.0 | MIT OR Apache-2.0 |
| `unicode-general-category` | 1.1.0 | Apache-2.0 |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `unicode-properties` | 0.1.5 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 |
| `untrusted` | 0.9.0 | ISC |
| `ureq` | 3.5.1 | MIT OR Apache-2.0 |
| `ureq-proto` | 0.7.1 | MIT OR Apache-2.0 |
| `url` | 2.5.8 | MIT OR Apache-2.0 |
| `urlencoding` | 2.1.3 | MIT |
| `utf8-zero` | 0.8.1 | MIT OR Apache-2.0 |
| `utf8_iter` | 1.1.0 | Apache-2.0 OR MIT |
| `utf_types` | 1.0.0 | Apache-2.0 OR MIT |
| `uuid` | 1.29.0 | Apache-2.0 OR MIT |
| `vello_common` | 0.2.0 | Apache-2.0 OR MIT |
| `vello_cpu` | 0.2.0 | Apache-2.0 OR MIT |
| `walkdir` | 2.5.0 | Unlicense/MIT |
| `want` | 0.3.2 | MIT |
| `wayland-backend` | 0.3.17 | MIT |
| `wayland-client` | 0.31.15 | MIT |
| `wayland-csd-frame` | 0.3.0 | MIT |
| `wayland-cursor` | 0.31.14 | MIT |
| `wayland-protocols` | 0.32.13 | MIT |
| `wayland-protocols-plasma` | 0.3.12 | MIT |
| `wayland-protocols-wlr` | 0.3.12 | MIT |
| `wayland-scanner` | 0.31.11 | MIT |
| `wayland-sys` | 0.31.11 | MIT |
| `web-time` | 1.1.0 | MIT OR Apache-2.0 |
| `webpki-roots` | 1.0.11 | CDLA-Permissive-2.0 |
| `winit` | 0.30.13 | Apache-2.0 |
| `winnow` | 1.0.5 | MIT |
| `writeable` | 0.6.4 | Unicode-3.0 |
| `x11-dl` | 2.21.0 | MIT |
| `x11rb` | 0.13.2 | MIT OR Apache-2.0 |
| `x11rb-protocol` | 0.13.2 | MIT OR Apache-2.0 |
| `xcursor` | 0.3.11 | MIT |
| `xkbcommon-dl` | 0.4.2 | MIT |
| `xkeysym` | 0.2.1 | MIT OR Apache-2.0 OR Zlib |
| `yoke` | 0.8.4 | Unicode-3.0 |
| `yoke-derive` | 0.8.4 | Unicode-3.0 |
| `zbus` | 5.19.0 | MIT |
| `zbus_macros` | 5.19.0 | MIT |
| `zbus_names` | 4.3.5 | MIT |
| `zcheapstr` | 1.2.0 | MIT |
| `zerocopy` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT |
| `zerofrom` | 0.1.8 | Unicode-3.0 |
| `zerofrom-derive` | 0.1.8 | Unicode-3.0 |
| `zeroize` | 1.9.1 | Apache-2.0 OR MIT |
| `zerotrie` | 0.2.5 | Unicode-3.0 |
| `zerovec` | 0.11.9 | Unicode-3.0 |
| `zerovec-derive` | 0.11.7 | Unicode-3.0 |
| `zip` | 9.4.0 | MIT |
| `zlib-rs` | 0.6.8 | Zlib |
| `zmij` | 1.0.23 | MIT |
| `zopfli` | 0.8.4 | Apache-2.0 |
| `zstd` | 0.14.3 | BSD-3-Clause |
| `zstd-safe` | 8.2.0 | BSD-3-Clause |
| `zstd-sys` | 2.1.2+zstd.1.5.7 | BSD-3-Clause |
| `zvariant` | 5.15.1 | MIT |
| `zvariant_derive` | 5.15.1 | MIT |
| `zvariant_utils` | 4.2.0 | MIT |
