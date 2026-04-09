pub type ThemeColor = (f64, f64, f64, f64);

// Background layers (deep → surface)
pub const BG_DEEP: ThemeColor = (0.118, 0.110, 0.094, 1.0);
pub const BG_PRIMARY: ThemeColor = (0.165, 0.153, 0.133, 1.0);
pub const BG_SECONDARY: ThemeColor = (0.204, 0.192, 0.173, 1.0);
pub const BG_ELEVATED: ThemeColor = (0.243, 0.231, 0.208, 1.0);
pub const BG_SURFACE: ThemeColor = (0.302, 0.286, 0.251, 1.0);

// Text & foreground
pub const TEXT_PRIMARY: ThemeColor = (0.855, 0.831, 0.745, 1.0);
pub const TEXT_SECONDARY: ThemeColor = (0.702, 0.675, 0.596, 1.0);
pub const TEXT_TERTIARY: ThemeColor = (0.549, 0.502, 0.416, 1.0);
pub const TEXT_ACCENT: ThemeColor = (0.922, 0.894, 0.824, 1.0);

// Pod program accents
pub const ACCENT_WARM: ThemeColor = (0.741, 0.616, 0.525, 1.0);
pub const ACCENT_GOLD: ThemeColor = (0.918, 0.875, 0.694, 1.0);
pub const ACCENT_CREAM: ThemeColor = (0.757, 0.702, 0.596, 1.0);

// Semantic (极度克制使用)
pub const STATUS_ACTIVE: ThemeColor = (0.478, 0.604, 0.420, 1.0);
pub const STATUS_WARNING: ThemeColor = (0.769, 0.627, 0.306, 1.0);
pub const STATUS_ERROR: ThemeColor = (0.627, 0.353, 0.290, 1.0);
pub const STATUS_INFO: ThemeColor = (0.416, 0.541, 0.604, 1.0);

// Borders & dividers
pub const BORDER_SUBTLE: ThemeColor = BG_ELEVATED;
pub const BORDER_STRONG: ThemeColor = BG_SURFACE;

// Disabled
pub const DISABLED_BG: ThemeColor = (0.180, 0.173, 0.157, 1.0);
pub const DISABLED_TEXT: ThemeColor = (0.376, 0.376, 0.376, 1.0);

// Typography scale
pub const TITLE_LG: f64 = 22.0;
pub const TITLE_MD: f64 = 16.0;
pub const TITLE_SM: f64 = 13.0;
pub const BODY: f64 = 12.0;
pub const CAPTION: f64 = 10.0;
pub const MICRO: f64 = 9.0;

pub const FONT_PRIMARY: &str = "Hiragino Sans";
pub const FONT_MONO: &str = "SF Mono";

pub fn hex_to_rgba(hex: &str) -> ThemeColor {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return (0.0, 0.0, 0.0, 1.0);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
    (r, g, b, 1.0)
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
mod platform {
    use super::ThemeColor;
    use objc::runtime::Object;

    /// # Safety
    /// `view` must be a valid, non-null pointer to an NSView or subclass.
    pub unsafe fn set_view_bg(view: *mut Object, color: ThemeColor) {
        unsafe {
            cocoanut::native::view_properties::set_background_color(
                view, color.0, color.1, color.2, color.3,
            );
        }
    }

    /// # Safety
    /// `view` must be a valid, non-null pointer to an NSTextField or similar control.
    pub unsafe fn set_text_color(view: *mut Object, color: ThemeColor) {
        unsafe {
            let ns_color: *mut Object = msg_send![
                class!(NSColor),
                colorWithSRGBRed: color.0
                green: color.1
                blue: color.2
                alpha: color.3
            ];
            let _: () = msg_send![view, setTextColor: ns_color];
        }
    }

    /// # Safety
    /// `view` must be a valid, non-null pointer to an NSControl that responds to `setFont:`.
    pub unsafe fn set_font(view: *mut Object, name: &str, size: f64) {
        let c_name = std::ffi::CString::new(name).unwrap_or_default();
        unsafe {
            let ns_name: *mut Object = msg_send![
                class!(NSString),
                stringWithUTF8String: c_name.as_ptr()
            ];
            let font: *mut Object =
                msg_send![class!(NSFont), fontWithName: ns_name size: size];
            if !font.is_null() {
                let _: () = msg_send![view, setFont: font];
            } else {
                let fallback: *mut Object =
                    msg_send![class!(NSFont), systemFontOfSize: size];
                let _: () = msg_send![view, setFont: fallback];
            }
        }
    }

    /// # Safety
    /// `window` must be a valid, non-null pointer to an NSWindow.
    pub unsafe fn apply_yorha_window(window: *mut Object) {
        let c_str = std::ffi::CString::new("NSAppearanceNameDarkAqua").unwrap_or_default();
        unsafe {
            let bg_color: *mut Object = msg_send![
                class!(NSColor),
                colorWithSRGBRed: super::BG_DEEP.0
                green: super::BG_DEEP.1
                blue: super::BG_DEEP.2
                alpha: super::BG_DEEP.3
            ];
            let _: () = msg_send![window, setBackgroundColor: bg_color];

            let appearance_name: *mut Object = msg_send![
                class!(NSString),
                stringWithUTF8String: c_str.as_ptr()
            ];
            let appearance: *mut Object =
                msg_send![class!(NSAppearance), appearanceNamed: appearance_name];
            if !appearance.is_null() {
                let _: () = msg_send![window, setAppearance: appearance];
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::ThemeColor;

    /// # Safety
    /// No-op stub on non-macOS. The `view` pointer is unused.
    pub unsafe fn set_view_bg(_view: *mut u8, _color: ThemeColor) {}
    /// # Safety
    /// No-op stub on non-macOS. The `view` pointer is unused.
    pub unsafe fn set_text_color(_view: *mut u8, _color: ThemeColor) {}
    /// # Safety
    /// No-op stub on non-macOS. The `view` pointer is unused.
    pub unsafe fn set_font(_view: *mut u8, _name: &str, _size: f64) {}
    /// # Safety
    /// No-op stub on non-macOS. The `window` pointer is unused.
    pub unsafe fn apply_yorha_window(_window: *mut u8) {}
}

pub use platform::*;

#[cfg(target_os = "macos")]
use cocoanut::prelude::*;

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicIsize, Ordering};

#[cfg(target_os = "macos")]
static NEXT_TAG: AtomicIsize = AtomicIsize::new(10000);

#[cfg(target_os = "macos")]
pub fn alloc_tag() -> isize {
    NEXT_TAG.fetch_add(1, Ordering::Relaxed)
}

#[cfg(target_os = "macos")]
pub fn yorha_group_box(title: &str) -> View {
    View::group_box(&title.to_uppercase())
}

#[cfg(target_os = "macos")]
pub fn yorha_button(label: &str) -> View {
    View::button(&label.to_uppercase())
}

#[cfg(target_os = "macos")]
pub fn yorha_stat_card(title: &str, value: &str, tag: isize) -> View {
    View::vstack()
        .child(View::text(value).bold().font_size(TITLE_LG).tag(tag))
        .child(
            View::text(&title.to_uppercase())
                .font_size(CAPTION),
        )
        .padding(8.0)
}

#[cfg(target_os = "macos")]
pub fn yorha_section_header(title: &str) -> View {
    View::vstack()
        .child(View::spacer().height(8.0))
        .child(View::text(&title.to_uppercase()).bold().font_size(MICRO))
        .child(View::spacer().height(4.0))
}

#[cfg(target_os = "macos")]
pub fn yorha_divider() -> View {
    View::spacer().height(1.0)
}

#[cfg(target_os = "macos")]
pub fn yorha_double_line() -> View {
    View::vstack()
        .child(View::spacer().height(1.0))
        .child(View::spacer().height(2.0))
        .child(View::spacer().height(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_to_rgba_parses_correctly() {
        let (r, g, b, a) = hex_to_rgba("#dad4be");
        assert!((r - 0.855).abs() < 0.005);
        assert!((g - 0.831).abs() < 0.005);
        assert!((b - 0.745).abs() < 0.005);
        assert!((a - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_to_rgba_handles_no_hash() {
        let (r, _, _, _) = hex_to_rgba("1e1c18");
        assert!((r - 0.118).abs() < 0.005);
    }

    #[test]
    fn hex_to_rgba_invalid_returns_black() {
        let c = hex_to_rgba("xyz");
        assert!((c.0).abs() < f64::EPSILON);
    }

    #[test]
    fn palette_colors_in_valid_range() {
        let colors = [
            BG_DEEP, BG_PRIMARY, BG_SECONDARY, BG_ELEVATED, BG_SURFACE,
            TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY, TEXT_ACCENT,
            ACCENT_WARM, ACCENT_GOLD, ACCENT_CREAM,
            STATUS_ACTIVE, STATUS_WARNING, STATUS_ERROR, STATUS_INFO,
            BORDER_SUBTLE, BORDER_STRONG, DISABLED_BG, DISABLED_TEXT,
        ];
        for (i, c) in colors.iter().enumerate() {
            assert!(c.0 >= 0.0 && c.0 <= 1.0, "color {i} red out of range");
            assert!(c.1 >= 0.0 && c.1 <= 1.0, "color {i} green out of range");
            assert!(c.2 >= 0.0 && c.2 <= 1.0, "color {i} blue out of range");
            assert!(c.3 >= 0.0 && c.3 <= 1.0, "color {i} alpha out of range");
        }
    }

    #[test]
    fn typography_constants_reasonable() {
        let sizes = [TITLE_LG, TITLE_MD, TITLE_SM, BODY, CAPTION, MICRO];
        for s in &sizes {
            assert!(*s > 0.0 && *s < 100.0);
        }
        assert!(TITLE_LG > TITLE_MD);
        assert!(TITLE_MD > TITLE_SM);
        assert!(TITLE_SM > BODY);
        assert!(BODY > CAPTION);
        assert!(CAPTION > MICRO);
    }
}
