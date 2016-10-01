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

pub mod system_fonts {
    use gdi32;
    use winapi::{c_int, c_void};
    use winapi::winnt::{PVOID, VOID};
    use winapi::wingdi::FIXED_PITCH;
    use winapi::wingdi::{ENUMLOGFONTEXW, LOGFONTW};
    use winapi::wingdi::FONTENUMPROCW;
    use winapi::minwindef::{DWORD, LPARAM};

    use std::ptr;
    use std::ffi::{OsStr, OsString};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    pub type FontProperty = LOGFONTW;

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
                    lfOutPrecision: 0,
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

		pub fn monospace() -> FontPropertyBuilder() {
			self.config.pfPitchAndFamily |= FIXED_PITCH;
		}
        // pub fn strikeout(mut self, strikeout: bool) -> FontConfigBuilder {
        // self.config.lfStrikeOut = strikeout as u8;
        // self
        // }
        //
        // pub fn underline(mut self, underline: bool) -> FontConfigBuilder {
        // self.config.lfUnderline = underline as u8;
        // self
        // }

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

    pub fn get(config: &FontProperty) -> Option<(Vec<u8>, c_int)> {
        unsafe {
            let hdc = gdi32::CreateCompatibleDC(ptr::null_mut());
            let hfont = gdi32::CreateFontIndirectW(config as *const LOGFONTW);
            gdi32::SelectObject(hdc, hfont as *mut c_void);
            let size = gdi32::GetFontData(hdc, 0, 0, ptr::null_mut(), 0);
            if size == 0xFFFFFFFF {
                gdi32::DeleteDC(hdc);
                None
            } else if size > 0 {
                let mut buffer: Vec<u8> = vec![0; size as usize];
                let pointer = buffer.first_mut().unwrap() as *mut _ as PVOID;
                let size = gdi32::GetFontData(hdc, 0, 0, pointer, size);
                buffer.set_len(size as usize);
                gdi32::DeleteDC(hdc);
                Some((buffer, 0))
            } else {
                gdi32::DeleteDC(hdc);
                None
            }
        }
    }

    pub fn query_all() -> Vec<String> {
        let mut config = FontPropertyBuilder::new().build();
        query(&mut config, Some(callback_ttf))
    }

    pub fn query_monospace() -> Vec<String> {
        let mut config = FontPropertyBuilder::new().build();
        query(&mut config, Some(callback_monospace))
    }

    pub fn query_specific(config: &mut FontProperty) -> Vec<String> {
        query(config, Some(callback_ttf))
    }


    fn query(lp_logfont: &mut LOGFONTW, f: FONTENUMPROCW) -> Vec<String> {

        let mut fonts = Vec::new();
        unsafe {
            let hdc = gdi32::CreateCompatibleDC(ptr::null_mut());

            let vec_pointer = &mut fonts as *mut Vec<String>;

            gdi32::EnumFontFamiliesExW(hdc, lp_logfont, f, vec_pointer as LPARAM, 0);
            gdi32::DeleteDC(hdc);
        }
        fonts
    }

    pub fn print_logfontw(l: &LOGFONTW) {
        // let name_2 = OsString::from_wide(&l.lfFaceName);
        // let name = name_2.to_str().unwrap();

        let name_array = l.lfFaceName;
        let pos = name_array.iter().position(|c| *c == 0).unwrap();
        let name_array = &name_array[0..pos];

        let name = OsString::from_wide(name_array).into_string().unwrap();
        println!("height:{} width:{} escapement:{} orientation:{} weight:{} italic:{} underline:{}
strikeout:{} charset:{} outprecision:{} clipprecision:{} quality:{} quality:{}
pichandfamily:{} ",
                 l.lfHeight,
                 l.lfWidth,
                 l.lfEscapement,
                 l.lfOrientation,
                 l.lfWeight,
                 l.lfItalic,
                 l.lfUnderline,
                 l.lfStrikeOut,
                 l.lfCharSet,
                 l.lfOutPrecision,
                 l.lfClipPrecision,
                 l.lfQuality,
                 l.lfPitchAndFamily,
                 name);
    }

    #[allow(non_snake_case)]
    unsafe extern "system" fn callback_ttf(lpelfe: *const LOGFONTW,
                                           _: *const VOID,
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
                                                 _: *const VOID,
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
}
