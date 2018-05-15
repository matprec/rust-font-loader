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
    use core_text::font_descriptor::*;
    use core_text::font_descriptor;
    use core_text;
    use std::fs::File;
    use std::mem;
    use std::ptr;
    use core_foundation::string::CFString;
    use core_foundation::number::CFNumber;
    use core_foundation::array::CFArray;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::url::CFURL;
    use libc::c_int;
    use std::io::Read;
    /// The platform specific font properties
    pub type FontProperty = CTFontDescriptor;

    /// Builder for FontProperty
    pub struct FontPropertyBuilder {
        symbolic_traits: CTFontSymbolicTraits,
        family: String
    }

    impl FontPropertyBuilder {
        pub fn new() -> FontPropertyBuilder {
            FontPropertyBuilder{ symbolic_traits: 0, family: String::new()}
        }

        pub fn italic(mut self) -> FontPropertyBuilder {
            self.symbolic_traits |= kCTFontItalicTrait;
            self
        }

        pub fn oblique(self) -> FontPropertyBuilder {
            self.italic()
        }

        pub fn monospace(mut self) -> FontPropertyBuilder {
            self.symbolic_traits |= kCTFontMonoSpaceTrait;
            self
        }

        pub fn bold(mut self) -> FontPropertyBuilder {
            self.symbolic_traits |= kCTFontBoldTrait;
            self
        }

        pub fn family(mut self, name: &str) -> FontPropertyBuilder {
            self.family = name.to_string();
            self
        }

        pub fn build(self) -> FontProperty {
            let family_attr: CFString = unsafe { TCFType::wrap_under_get_rule(kCTFontFamilyNameAttribute) };
            let family_name: CFString = self.family.parse().unwrap();
            let traits_attr: CFString = unsafe { TCFType::wrap_under_get_rule(kCTFontTraitsAttribute) };
            let symbolic_traits_attr: CFString = unsafe { TCFType::wrap_under_get_rule(kCTFontSymbolicTrait) };
            let traits = CFDictionary::from_CFType_pairs(&[(symbolic_traits_attr.as_CFType(), CFNumber::from(self.symbolic_traits as i32).as_CFType())]);
            let mut attributes = Vec::new();
            attributes.push((traits_attr, traits.as_CFType()));
            if self.family.len() != 0 {
                attributes.push((family_attr, family_name.as_CFType()));
            }
            let attributes = CFDictionary::from_CFType_pairs(&attributes);
            font_descriptor::new_from_attributes(&attributes)
        }
    }

    /// Get the binary data and index of a specific font
    pub fn get(config: &FontProperty) -> Option<(Vec<u8>, c_int)> {
        let mut buffer = Vec::new();
        let url: CFURL;
        unsafe {
            let value =
                CTFontDescriptorCopyAttribute(config.as_concrete_TypeRef(), kCTFontURLAttribute);

            if value.is_null() {
                return None
            }

            let value: CFType = TCFType::wrap_under_get_rule(value);
            if !value.instance_of::<CFURL>() {
                return None
            }
            url = TCFType::wrap_under_get_rule(mem::transmute(value.as_CFTypeRef()));
        }
        if let Some(path) = url.to_path() {
            match File::open(path).and_then(|mut f| f.read_to_end(&mut buffer)) {
                Ok(_) => return Some((buffer, 0)),
                Err(_) => return None,
            }
        };
        return None
    }

    /// Query the names of all fonts installed in the system
    pub fn query_all() -> Vec<String> {
        core_text::font_collection::get_family_names()
            .iter()
            .map(|family_name| family_name.to_string())
            .collect()
    }

    /// Query the names of specifc fonts installed in the system
    pub fn query_specific(property: &mut FontProperty) -> Vec<String> {
        let descs: CFArray<CTFontDescriptor> = unsafe {
            let descs = CTFontDescriptorCreateMatchingFontDescriptors(
                property.as_concrete_TypeRef(),
                ptr::null(),
            );
            TCFType::wrap_under_create_rule(descs)
        };
        descs
            .iter()
            .map(|desc| desc.family_name())
            .collect::<Vec<_>>()
    }
}
