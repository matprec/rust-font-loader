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

/// Font loading utilities for installed system fonts
pub mod system_fonts {
    use winapi::um::wingdi;
    use winapi::um::wingdi::TEXTMETRICW;
    use winapi::ctypes::{c_int, c_void};
    use winapi::um::winnt::{PVOID};
    use winapi::um::wingdi::FIXED_PITCH;
    use winapi::um::wingdi::{ENUMLOGFONTEXW, LOGFONTW, OUT_TT_ONLY_PRECIS};
    use winapi::um::wingdi::FONTENUMPROCW;
    use winapi::shared::minwindef::{DWORD, LPARAM};

    use std::ptr;
    use std::mem;
    use std::ffi::{OsStr, OsString};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    /// The platform specific font properties
    pub type FontProperty = LOGFONTW;

    /// Builder for FontProperty
    pub struct FontPropertyBuilder {
        config: FontProperty,
    }

    impl FontPropertyBuilder {
        pub fn new() -> FontPropertyBuilder {
            let string: [u16; 32] = [0; 32];
            FontPropertyBuilder {
                config: FontProperty {
                    lfHeight: 0,
                    lfWidth: 0,
                    lfEscapement: 0,
                    lfOrientation: 0,
                    lfWeight: 0,
                    lfItalic: 0,
                    lfUnderline: 0,
                    lfStrikeOut: 0,
                    lfCharSet: 0,
                    lfOutPrecision: OUT_TT_ONLY_PRECIS as u8,
                    lfClipPrecision: 0,
                    lfQuality: 0,
                    lfPitchAndFamily: 0,
                    lfFaceName: string,
                },
            }
        }

        pub fn italic(mut self) -> FontPropertyBuilder {
            self.config.lfItalic = true as u8;
            self
        }

        pub fn oblique(self) -> FontPropertyBuilder {
            self.italic()
        }

        pub fn monospace(mut self) -> FontPropertyBuilder {
            self.config.lfPitchAndFamily |= FIXED_PITCH as u8;
            self
        }

        pub fn bold(mut self) -> FontPropertyBuilder {
            self.config.lfWeight = 700;
            self
        }

        pub fn family(mut self, name: &str) -> FontPropertyBuilder {
            if name.len() > 31 {
                panic!("Font length must me smaller than 31");
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

        pub fn build(self) -> FontProperty {
            self.config
        }
    }

    /// Get the binary data and index of a specific font
    /// Note that only truetype fonts are supported
    pub fn get(config: &FontProperty) -> Option<(Vec<u8>, c_int)> {
        unsafe {
            let hdc = wingdi::CreateCompatibleDC(ptr::null_mut());
            let hfont = wingdi::CreateFontIndirectW(config as *const LOGFONTW);
            wingdi::SelectObject(hdc, hfont as *mut c_void);
            let size = wingdi::GetFontData(hdc, 0, 0, ptr::null_mut(), 0);
            if size == 0xFFFFFFFF {
                wingdi::DeleteDC(hdc);
                None
            } else if size > 0 {
                let mut buffer: Vec<u8> = vec![0; size as usize];
                let pointer = buffer.first_mut().unwrap() as *mut _ as PVOID;
                let size = wingdi::GetFontData(hdc, 0, 0, pointer, size);
                buffer.set_len(size as usize);
                wingdi::DeleteDC(hdc);
                Some((buffer, 0))
            } else {
                wingdi::DeleteDC(hdc);
                None
            }
        }
    }

    pub fn get_native(config: &mut FontProperty) -> FontProperty {
        let f: FONTENUMPROCW = Some(callback_native);
        unsafe {
            let mut logfont: LOGFONTW = mem::zeroed();
            let pointer = &mut logfont as *mut _;
            let hdc = wingdi::CreateCompatibleDC(ptr::null_mut());
            wingdi::EnumFontFamiliesExW(hdc, config, f, pointer as LPARAM, 0);
            wingdi::DeleteDC(hdc);
            logfont
        }
    }

    /// Query the names of all fonts installed in the system
    /// Note that only truetype fonts are supported
    pub fn query_all() -> Vec<String> {
        let mut config = FontPropertyBuilder::new().build();
        query_specific(&mut config)
    }

    /// Query the names of specifc fonts installed in the system
    /// Note that only truetype fonts are supported
    pub fn query_specific(property: &mut FontProperty) -> Vec<String> {

        let mut fonts = Vec::new();
        let mut f: FONTENUMPROCW = Some(callback_ttf);
        unsafe {
            let hdc = wingdi::CreateCompatibleDC(ptr::null_mut());

            if (property.lfPitchAndFamily & FIXED_PITCH as u8) != 0 {
                f = Some(callback_monospace);
            }

            let vec_pointer = &mut fonts as *mut Vec<String>;

            wingdi::EnumFontFamiliesExW(hdc, property, f, vec_pointer as LPARAM, 0);
            wingdi::DeleteDC(hdc);
        }
        fonts
    }

    #[allow(non_snake_case)]
    unsafe extern "system" fn callback_ttf(lpelfe: *const LOGFONTW,
                                           _: *const TEXTMETRICW,
                                           fonttype: DWORD,
                                           lparam: LPARAM)
                                           -> c_int {

        if fonttype != 4 {
            return 1;
        }

        add_vec(lpelfe, lparam);

        1
    }

    #[allow(non_snake_case)]
    unsafe extern "system" fn callback_monospace(lpelfe: *const LOGFONTW,
                                                 _: *const TEXTMETRICW,
                                                 fonttype: DWORD,
                                                 lparam: LPARAM)
                                                 -> c_int {
        if fonttype != 4 {
            return 1;
        }

        if ((*lpelfe).lfPitchAndFamily & FIXED_PITCH as u8) == 0 {
            return 1;
        }
        add_vec(lpelfe, lparam);

        1
    }

    unsafe fn add_vec(lpelfe: *const LOGFONTW, lparam: LPARAM) {
        let lpelfe = lpelfe as *const ENUMLOGFONTEXW;

        let name_array = (*lpelfe).elfFullName;
        let pos = name_array.iter().position(|c| *c == 0).unwrap();
        let name_array = &name_array[0..pos];

        let name = OsString::from_wide(name_array).into_string().unwrap();

        if name.chars().next() != Some('@') {
            let vec_pointer = lparam as *mut Vec<String>;
            let ref mut fonts = *vec_pointer;
            fonts.push(name);
        }
    }

    #[allow(non_snake_case)]
    unsafe extern "system" fn callback_native(lpelfe: *const LOGFONTW,
                                              _: *const TEXTMETRICW,
                                              fonttype: DWORD,
                                              lparam: LPARAM)
                                              -> c_int {

        if fonttype != 4 {
            return 1;
        }

        ptr::copy(lpelfe, lparam as *mut _, 1);

        0
    }

}
