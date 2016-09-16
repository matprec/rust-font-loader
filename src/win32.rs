// The MIT License (MIT)
// Copyright (c) font-loader Developers
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

use gdi32;
use winapi::{c_int, c_void};
use winapi::winnt::{PVOID, VOID};
use winapi::wingdi::{ANSI_CHARSET, CLIP_DEFAULT_PRECIS, DEFAULT_PITCH, DEFAULT_QUALITY};
use winapi::wingdi::{ENUMLOGFONTEXW, FF_DONTCARE, LOGFONTW, OUT_DEFAULT_PRECIS};
use winapi::minwindef::{DWORD, LPARAM};

use std::{mem, ptr};
use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};

pub type FontConfig = LOGFONTW;

pub struct FontConfigBuilder {
    config: FontConfig,
}

impl FontConfigBuilder {
    pub fn new() -> FontConfigBuilder {
        let string: [u16; 32] = [0; 32];
        FontConfigBuilder { config: FontConfig { lfFaceName: string, ..unsafe { mem::zeroed() } } }
    }

    pub fn italic(mut self, italic: bool) -> FontConfigBuilder {
        self.config.lfItalic = italic as u8;
        self
    }

    pub fn strikeout(mut self, strikeout: bool) -> FontConfigBuilder {
        self.config.lfStrikeOut = strikeout as u8;
        self
    }

    pub fn underline(mut self, underline: bool) -> FontConfigBuilder {
        self.config.lfUnderline = underline as u8;
        self
    }

    pub fn weight(mut self, weight: usize) -> FontConfigBuilder {
        self.config.lfWeight = weight as i32;
        self
    }

    pub fn facename(mut self, name: &str) -> FontConfigBuilder {
        if name.len() > 31 {
            panic!("Size must me smaller than 31");
        }
        let name: &OsStr = name.as_ref();
        let buffer = name.encode_wide();
        let mut string: [u16; 32] = [0; 32]; // +1 Null terminator
        for (index, item) in buffer.enumerate() {
            string[index] = item;
        }
        self.config.lfFaceName = string;
        self
    }

    pub fn build(self) -> FontConfig {
        self.config
    }
}

pub struct SystemFonts {
    fonts: Vec<String>,
}

impl SystemFonts {
    pub fn new() -> SystemFonts {
        SystemFonts { fonts: Vec::new() }
    }

    pub fn font_by_name(&self, config: FontConfig) -> Result<Vec<u8>, ()> {
        unsafe {
            let hdc = gdi32::CreateCompatibleDC(ptr::null_mut());
            let hfont = gdi32::CreateFontIndirectW(&config as *const LOGFONTW);
            gdi32::SelectObject(hdc, hfont as *mut c_void);
            let size = gdi32::GetFontData(hdc, 0, 0, ptr::null_mut(), 0);
            println!("size: {}", size);
            if size == 0xFFFFFFFF {
                Err(())
            } else if size > 0 {
                let mut buffer: Vec<u8> = vec![0; size as usize];
                let pointer = buffer.first_mut().unwrap() as *mut _ as PVOID;
                let size = gdi32::GetFontData(hdc, 0, 0, pointer, size);
                buffer.set_len(size as usize);
                gdi32::DeleteDC(hdc);
                Ok(buffer)
            } else {
                gdi32::DeleteDC(hdc);
                Err(())
            }
        }
    }

    #[allow(non_snake_case)]
    pub fn enumerate_fonts(&self) -> &Vec<String> {
        unsafe {
            let hdc = gdi32::CreateCompatibleDC(ptr::null_mut());
            let string: [u16; 32] = [0; 32];

            let mut lpLogfont = LOGFONTW {
                lfHeight: 0,
                lfWidth: 0,
                lfEscapement: 0,
                lfOrientation: 0,
                lfWeight: 0,
                lfItalic: false as u8,
                lfUnderline: false as u8,
                lfStrikeOut: false as u8,
                lfCharSet: ANSI_CHARSET as u8,
                lfOutPrecision: OUT_DEFAULT_PRECIS as u8,
                lfClipPrecision: CLIP_DEFAULT_PRECIS as u8,
                lfQuality: DEFAULT_QUALITY as u8,
                lfPitchAndFamily: (DEFAULT_PITCH | FF_DONTCARE) as u8,
                lfFaceName: string,
            };

            let self_pointer = self as *const SystemFonts;

            gdi32::EnumFontFamiliesExW(hdc,
                                       &mut lpLogfont,
                                       Some(SystemFonts::callback),
                                       self_pointer as LPARAM,
                                       0);
            gdi32::DeleteDC(hdc);
        }
        &self.fonts
    }

    #[allow(non_snake_case)]
    unsafe extern "system" fn callback(lpelfe: *const LOGFONTW,
                                       _: *const VOID,
                                       _: DWORD,
                                       lparam: LPARAM)
                                       -> c_int {
        let lpelfe = lpelfe as *const ENUMLOGFONTEXW;

        let name_array = (*lpelfe).elfFullName;
        let pos = name_array.iter().position(|c| *c == 0).unwrap();
        let name_array = &name_array[0..pos];

        let name = OsString::from_wide(name_array).into_string().unwrap();

        if name.chars().next() != Some('@') {
            let systemfonts = lparam as *mut SystemFonts;
            let ref mut systemfonts = *systemfonts;
            systemfonts.fonts.push(name);
        }

        1
    }
}
