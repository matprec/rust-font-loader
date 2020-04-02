// The MIT License (MIT)
// Copyright (c) 2016 font-loader Developers
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
// associated documentation files (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify, merge, publish, distribute,
// sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
// NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! # Font-Loader
//! A font loading utility written in and for Rust.
//!
//! Currently supported platforms:
//!
//! * Windows
//! * Mac
//! * *nix systems
//!
//! # Usage
//! ## Linux, Unix:
//! Fontconfig is required on Linux and Unix, as it is the default Fontmanagement utility on these
//! platforms.
//!
//! ```shell
//! sudo apt-get install libfontconfig libfontconfig1-dev
//! ```
//!
//! # Example
//! ## Cargo.toml
//! ```toml
//! [dependencies]
//! font-loader = "https://github.com/matprec/rust-font-loader"
//! ```
//!
//! ## main.rs:
//! ```rust
//! extern crate font_loader as fonts;
//!
//! use fonts::system_fonts;
//!
//! fn main() {
//! 	// Enumerate all fonts
//!     let sysfonts = system_fonts::query_all();
//!     for string in &sysfonts {
//!         println!("{}", string);
//!     }
//!
//! 	let mut property = system_fonts::FontPropertyBuilder::new().monospace().build();
//! 	let sysfonts = system_fonts::query_specific(&mut property);
//! 	for string in &sysfonts {
//! 		println!("Monospaced font: {}", string);
//! 	}
//!
//! 	let property = system_fonts::FontPropertyBuilder::new().family("Arial").build();
//! 	let (font, _) = system_fonts::get(&property).unwrap();
//! 	println!("{:?}", &font[..50]);
//! }
//! ```


extern crate libc;

#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(target_os = "windows")]
mod win32;
#[cfg(target_os = "windows")]
pub use win32::*;

#[cfg(target_os = "macos")]
extern crate core_text;
#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(all(unix, not(target_os = "macos")))]
extern crate fontconfig as servo_fontconfig;
#[cfg(all(unix, not(target_os = "macos")))]
mod fontconfig;
#[cfg(all(unix, not(target_os = "macos")))]
pub use fontconfig::*;
