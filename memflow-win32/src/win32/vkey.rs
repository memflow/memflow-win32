use core::ops::{Deref, DerefMut};

/// Windows Virtual Key Codes
/// Based on the windows rust api https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Input/KeyboardAndMouse/
/// except more flexible and cross platform
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VKEY(pub u16);

/// auto implement Display based on actual enum name
/// Utilizes the Debug trait to print the enum name
/// This works better on enum types.
impl std::fmt::Display for VKEY {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Deref for VKEY {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for VKEY {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<VKEY> for u16 {
    fn from(vk: VKEY) -> Self {
        vk.0
    }
}
impl From<u16> for VKEY {
    fn from(v: u16) -> Self {
        VKEY(v)
    }
}
impl From<VKEY> for u32 {
    fn from(vk: VKEY) -> Self {
        vk.0 as u32
    }
}
impl From<u32> for VKEY {
    fn from(v: u32) -> Self {
        VKEY(v as u16)
    }
}
impl From<i32> for VKEY {
    fn from(v: i32) -> Self {
        VKEY(v as u16)
    }
}
impl From<VKEY> for i32 {
    fn from(vk: VKEY) -> Self {
        vk.0 as i32
    }
}

/// Iterate over a range of VKEYs (inclusive start, exclusive end)
/// Placeholder until we can use the `Step` trait
/// in a stable way
pub fn vkey_range(start: VKEY, end: VKEY) -> impl Iterator<Item = VKEY> {
    (start.0..end.0).map(VKEY)
}

// #[cfg(all(feature = "nightly", nightly))]
// use std::iter::Step;
// #[cfg(all(feature = "nightly", nightly))]
// impl std::iter::Step for VKEY {
//     fn steps_between(start: &Self, end: &Self) -> Option<usize> {
//         u16::steps_between(&start.0, &end.0)
//     }
//     fn forward_checked(start: Self, count: usize) -> Option<Self> {
//         u16::forward_checked(start.0, count).map(VKEY)
//     }
//     fn backward_checked(start: Self, count: usize) -> Option<Self> {
//         u16::backward_checked(start.0, count).map(VKEY)
//     }
// }

pub const VK_0: VKEY = VKEY(48u16);
pub const VK_1: VKEY = VKEY(49u16);
pub const VK_2: VKEY = VKEY(50u16);
pub const VK_3: VKEY = VKEY(51u16);
pub const VK_4: VKEY = VKEY(52u16);
pub const VK_5: VKEY = VKEY(53u16);
pub const VK_6: VKEY = VKEY(54u16);
pub const VK_7: VKEY = VKEY(55u16);
pub const VK_8: VKEY = VKEY(56u16);
pub const VK_9: VKEY = VKEY(57u16);
pub const VK_A: VKEY = VKEY(65u16);
pub const VK_ABNT_C1: VKEY = VKEY(193u16);
pub const VK_ABNT_C2: VKEY = VKEY(194u16);
pub const VK_ACCEPT: VKEY = VKEY(30u16);
pub const VK_ADD: VKEY = VKEY(107u16);
pub const VK_APPS: VKEY = VKEY(93u16);
pub const VK_ATTN: VKEY = VKEY(246u16);
pub const VK_B: VKEY = VKEY(66u16);
pub const VK_BACK: VKEY = VKEY(8u16);
pub const VK_BROWSER_BACK: VKEY = VKEY(166u16);
pub const VK_BROWSER_FAVORITES: VKEY = VKEY(171u16);
pub const VK_BROWSER_FORWARD: VKEY = VKEY(167u16);
pub const VK_BROWSER_HOME: VKEY = VKEY(172u16);
pub const VK_BROWSER_REFRESH: VKEY = VKEY(168u16);
pub const VK_BROWSER_SEARCH: VKEY = VKEY(170u16);
pub const VK_BROWSER_STOP: VKEY = VKEY(169u16);
pub const VK_C: VKEY = VKEY(67u16);
pub const VK_CANCEL: VKEY = VKEY(3u16);
pub const VK_CAPITAL: VKEY = VKEY(20u16);
pub const VK_CLEAR: VKEY = VKEY(12u16);
pub const VK_CONTROL: VKEY = VKEY(17u16);
pub const VK_CONVERT: VKEY = VKEY(28u16);
pub const VK_CRSEL: VKEY = VKEY(247u16);
pub const VK_D: VKEY = VKEY(68u16);
pub const VK_DBE_ALPHANUMERIC: VKEY = VKEY(240u16);
pub const VK_DBE_CODEINPUT: VKEY = VKEY(250u16);
pub const VK_DBE_DBCSCHAR: VKEY = VKEY(244u16);
pub const VK_DBE_DETERMINESTRING: VKEY = VKEY(252u16);
pub const VK_DBE_ENTERDLGCONVERSIONMODE: VKEY = VKEY(253u16);
pub const VK_DBE_ENTERIMECONFIGMODE: VKEY = VKEY(248u16);
pub const VK_DBE_ENTERWORDREGISTERMODE: VKEY = VKEY(247u16);
pub const VK_DBE_FLUSHSTRING: VKEY = VKEY(249u16);
pub const VK_DBE_HIRAGANA: VKEY = VKEY(242u16);
pub const VK_DBE_KATAKANA: VKEY = VKEY(241u16);
pub const VK_DBE_NOCODEINPUT: VKEY = VKEY(251u16);
pub const VK_DBE_NOROMAN: VKEY = VKEY(246u16);
pub const VK_DBE_ROMAN: VKEY = VKEY(245u16);
pub const VK_DBE_SBCSCHAR: VKEY = VKEY(243u16);
pub const VK_DECIMAL: VKEY = VKEY(110u16);
pub const VK_DELETE: VKEY = VKEY(46u16);
pub const VK_DIVIDE: VKEY = VKEY(111u16);
pub const VK_DOWN: VKEY = VKEY(40u16);
pub const VK_E: VKEY = VKEY(69u16);
pub const VK_END: VKEY = VKEY(35u16);
pub const VK_EREOF: VKEY = VKEY(249u16);
pub const VK_ESCAPE: VKEY = VKEY(27u16);
pub const VK_EXECUTE: VKEY = VKEY(43u16);
pub const VK_EXSEL: VKEY = VKEY(248u16);

pub const VK_F: VKEY = VKEY(70u16);
pub const VK_F1: VKEY = VKEY(112u16);
pub const VK_F10: VKEY = VKEY(121u16);
pub const VK_F11: VKEY = VKEY(122u16);
pub const VK_F12: VKEY = VKEY(123u16);
pub const VK_F13: VKEY = VKEY(124u16);
pub const VK_F14: VKEY = VKEY(125u16);
pub const VK_F15: VKEY = VKEY(126u16);
pub const VK_F16: VKEY = VKEY(127u16);
pub const VK_F17: VKEY = VKEY(128u16);
pub const VK_F18: VKEY = VKEY(129u16);
pub const VK_F19: VKEY = VKEY(130u16);
pub const VK_F2: VKEY = VKEY(113u16);
pub const VK_F20: VKEY = VKEY(131u16);
pub const VK_F21: VKEY = VKEY(132u16);
pub const VK_F22: VKEY = VKEY(133u16);
pub const VK_F23: VKEY = VKEY(134u16);
pub const VK_F24: VKEY = VKEY(135u16);
pub const VK_F3: VKEY = VKEY(114u16);
pub const VK_F4: VKEY = VKEY(115u16);
pub const VK_F5: VKEY = VKEY(116u16);
pub const VK_F6: VKEY = VKEY(117u16);
pub const VK_F7: VKEY = VKEY(118u16);
pub const VK_F8: VKEY = VKEY(119u16);
pub const VK_F9: VKEY = VKEY(120u16);
pub const VK_FINAL: VKEY = VKEY(24u16);

pub const VK_G: VKEY = VKEY(71u16);
pub const VK_GAMEPAD_A: VKEY = VKEY(195u16);
pub const VK_GAMEPAD_B: VKEY = VKEY(196u16);
pub const VK_GAMEPAD_DPAD_DOWN: VKEY = VKEY(204u16);
pub const VK_GAMEPAD_DPAD_LEFT: VKEY = VKEY(205u16);
pub const VK_GAMEPAD_DPAD_RIGHT: VKEY = VKEY(206u16);
pub const VK_GAMEPAD_DPAD_UP: VKEY = VKEY(203u16);
pub const VK_GAMEPAD_LEFT_SHOULDER: VKEY = VKEY(200u16);
pub const VK_GAMEPAD_LEFT_THUMBSTICK_BUTTON: VKEY = VKEY(209u16);
pub const VK_GAMEPAD_LEFT_THUMBSTICK_DOWN: VKEY = VKEY(212u16);
pub const VK_GAMEPAD_LEFT_THUMBSTICK_LEFT: VKEY = VKEY(214u16);
pub const VK_GAMEPAD_LEFT_THUMBSTICK_RIGHT: VKEY = VKEY(213u16);
pub const VK_GAMEPAD_LEFT_THUMBSTICK_UP: VKEY = VKEY(211u16);
pub const VK_GAMEPAD_LEFT_TRIGGER: VKEY = VKEY(201u16);
pub const VK_GAMEPAD_MENU: VKEY = VKEY(207u16);
pub const VK_GAMEPAD_RIGHT_SHOULDER: VKEY = VKEY(199u16);
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_BUTTON: VKEY = VKEY(210u16);
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_DOWN: VKEY = VKEY(216u16);
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_LEFT: VKEY = VKEY(218u16);
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_RIGHT: VKEY = VKEY(217u16);
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_UP: VKEY = VKEY(215u16);
pub const VK_GAMEPAD_RIGHT_TRIGGER: VKEY = VKEY(202u16);
pub const VK_GAMEPAD_VIEW: VKEY = VKEY(208u16);
pub const VK_GAMEPAD_X: VKEY = VKEY(197u16);
pub const VK_GAMEPAD_Y: VKEY = VKEY(198u16);
pub const VK_H: VKEY = VKEY(72u16);
pub const VK_HANGEUL: VKEY = VKEY(21u16);
pub const VK_HANGUL: VKEY = VKEY(21u16);
pub const VK_HANJA: VKEY = VKEY(25u16);
pub const VK_HELP: VKEY = VKEY(47u16);
pub const VK_HOME: VKEY = VKEY(36u16);
pub const VK_I: VKEY = VKEY(73u16);
pub const VK_ICO_00: VKEY = VKEY(228u16);
pub const VK_ICO_CLEAR: VKEY = VKEY(230u16);
pub const VK_ICO_HELP: VKEY = VKEY(227u16);
pub const VK_IME_OFF: VKEY = VKEY(26u16);
pub const VK_IME_ON: VKEY = VKEY(22u16);
pub const VK_INSERT: VKEY = VKEY(45u16);
pub const VK_J: VKEY = VKEY(74u16);
pub const VK_JUNJA: VKEY = VKEY(23u16);
pub const VK_K: VKEY = VKEY(75u16);
pub const VK_KANA: VKEY = VKEY(21u16);
pub const VK_KANJI: VKEY = VKEY(25u16);
pub const VK_L: VKEY = VKEY(76u16);
pub const VK_LAUNCH_APP1: VKEY = VKEY(182u16);
pub const VK_LAUNCH_APP2: VKEY = VKEY(183u16);
pub const VK_LAUNCH_MAIL: VKEY = VKEY(180u16);
pub const VK_LAUNCH_MEDIA_SELECT: VKEY = VKEY(181u16);
pub const VK_LBUTTON: VKEY = VKEY(1u16);
pub const VK_LCONTROL: VKEY = VKEY(162u16);
pub const VK_LEFT: VKEY = VKEY(37u16);
pub const VK_LMENU: VKEY = VKEY(164u16);
pub const VK_LSHIFT: VKEY = VKEY(160u16);
pub const VK_LWIN: VKEY = VKEY(91u16);
pub const VK_M: VKEY = VKEY(77u16);
pub const VK_MBUTTON: VKEY = VKEY(4u16);
pub const VK_MEDIA_NEXT_TRACK: VKEY = VKEY(176u16);
pub const VK_MEDIA_PLAY_PAUSE: VKEY = VKEY(179u16);
pub const VK_MEDIA_PREV_TRACK: VKEY = VKEY(177u16);
pub const VK_MEDIA_STOP: VKEY = VKEY(178u16);
pub const VK_MENU: VKEY = VKEY(18u16);
pub const VK_MODECHANGE: VKEY = VKEY(31u16);
pub const VK_MULTIPLY: VKEY = VKEY(106u16);
pub const VK_N: VKEY = VKEY(78u16);
pub const VK_NAVIGATION_ACCEPT: VKEY = VKEY(142u16);
pub const VK_NAVIGATION_CANCEL: VKEY = VKEY(143u16);
pub const VK_NAVIGATION_DOWN: VKEY = VKEY(139u16);
pub const VK_NAVIGATION_LEFT: VKEY = VKEY(140u16);
pub const VK_NAVIGATION_MENU: VKEY = VKEY(137u16);
pub const VK_NAVIGATION_RIGHT: VKEY = VKEY(141u16);
pub const VK_NAVIGATION_UP: VKEY = VKEY(138u16);
pub const VK_NAVIGATION_VIEW: VKEY = VKEY(136u16);
pub const VK_NEXT: VKEY = VKEY(34u16);
pub const VK_NONAME: VKEY = VKEY(252u16);
pub const VK_NONCONVERT: VKEY = VKEY(29u16);
pub const VK_NUMLOCK: VKEY = VKEY(144u16);
pub const VK_NUMPAD0: VKEY = VKEY(96u16);
pub const VK_NUMPAD1: VKEY = VKEY(97u16);
pub const VK_NUMPAD2: VKEY = VKEY(98u16);
pub const VK_NUMPAD3: VKEY = VKEY(99u16);
pub const VK_NUMPAD4: VKEY = VKEY(100u16);
pub const VK_NUMPAD5: VKEY = VKEY(101u16);
pub const VK_NUMPAD6: VKEY = VKEY(102u16);
pub const VK_NUMPAD7: VKEY = VKEY(103u16);
pub const VK_NUMPAD8: VKEY = VKEY(104u16);
pub const VK_NUMPAD9: VKEY = VKEY(105u16);
pub const VK_O: VKEY = VKEY(79u16);
pub const VK_OEM_1: VKEY = VKEY(186u16);
pub const VK_OEM_102: VKEY = VKEY(226u16);
pub const VK_OEM_2: VKEY = VKEY(191u16);
pub const VK_OEM_3: VKEY = VKEY(192u16);
pub const VK_OEM_4: VKEY = VKEY(219u16);
pub const VK_OEM_5: VKEY = VKEY(220u16);
pub const VK_OEM_6: VKEY = VKEY(221u16);
pub const VK_OEM_7: VKEY = VKEY(222u16);
pub const VK_OEM_8: VKEY = VKEY(223u16);
pub const VK_OEM_ATTN: VKEY = VKEY(240u16);
pub const VK_OEM_AUTO: VKEY = VKEY(243u16);
pub const VK_OEM_AX: VKEY = VKEY(225u16);
pub const VK_OEM_BACKTAB: VKEY = VKEY(245u16);
pub const VK_OEM_CLEAR: VKEY = VKEY(254u16);
pub const VK_OEM_COMMA: VKEY = VKEY(188u16);
pub const VK_OEM_COPY: VKEY = VKEY(242u16);
pub const VK_OEM_CUSEL: VKEY = VKEY(239u16);
pub const VK_OEM_ENLW: VKEY = VKEY(244u16);
pub const VK_OEM_FINISH: VKEY = VKEY(241u16);
pub const VK_OEM_FJ_JISHO: VKEY = VKEY(146u16);
pub const VK_OEM_FJ_LOYA: VKEY = VKEY(149u16);
pub const VK_OEM_FJ_MASSHOU: VKEY = VKEY(147u16);
pub const VK_OEM_FJ_ROYA: VKEY = VKEY(150u16);
pub const VK_OEM_FJ_TOUROKU: VKEY = VKEY(148u16);
pub const VK_OEM_JUMP: VKEY = VKEY(234u16);
pub const VK_OEM_MINUS: VKEY = VKEY(189u16);
pub const VK_OEM_NEC_EQUAL: VKEY = VKEY(146u16);
pub const VK_OEM_PA1: VKEY = VKEY(235u16);
pub const VK_OEM_PA2: VKEY = VKEY(236u16);
pub const VK_OEM_PA3: VKEY = VKEY(237u16);
pub const VK_OEM_PERIOD: VKEY = VKEY(190u16);
pub const VK_OEM_PLUS: VKEY = VKEY(187u16);
pub const VK_OEM_RESET: VKEY = VKEY(233u16);
pub const VK_OEM_WSCTRL: VKEY = VKEY(238u16);
pub const VK_P: VKEY = VKEY(80u16);
pub const VK_PA1: VKEY = VKEY(253u16);
pub const VK_PACKET: VKEY = VKEY(231u16);
pub const VK_PAUSE: VKEY = VKEY(19u16);
pub const VK_PLAY: VKEY = VKEY(250u16);
pub const VK_PRINT: VKEY = VKEY(42u16);
pub const VK_PRIOR: VKEY = VKEY(33u16);
pub const VK_PROCESSKEY: VKEY = VKEY(229u16);
pub const VK_Q: VKEY = VKEY(81u16);
pub const VK_R: VKEY = VKEY(82u16);
pub const VK_RBUTTON: VKEY = VKEY(2u16);
pub const VK_RCONTROL: VKEY = VKEY(163u16);
pub const VK_RETURN: VKEY = VKEY(13u16);
pub const VK_RIGHT: VKEY = VKEY(39u16);
pub const VK_RMENU: VKEY = VKEY(165u16);
pub const VK_RSHIFT: VKEY = VKEY(161u16);
pub const VK_RWIN: VKEY = VKEY(92u16);
pub const VK_S: VKEY = VKEY(83u16);
pub const VK_SCROLL: VKEY = VKEY(145u16);
pub const VK_SELECT: VKEY = VKEY(41u16);
pub const VK_SEPARATOR: VKEY = VKEY(108u16);
pub const VK_SHIFT: VKEY = VKEY(16u16);
pub const VK_SLEEP: VKEY = VKEY(95u16);
pub const VK_SNAPSHOT: VKEY = VKEY(44u16);
pub const VK_SPACE: VKEY = VKEY(32u16);
pub const VK_SUBTRACT: VKEY = VKEY(109u16);
pub const VK_T: VKEY = VKEY(84u16);
pub const VK_TAB: VKEY = VKEY(9u16);

pub const VK_U: VKEY = VKEY(85u16);
pub const VK_UP: VKEY = VKEY(38u16);
pub const VK_V: VKEY = VKEY(86u16);
pub const VK_VOLUME_DOWN: VKEY = VKEY(174u16);
pub const VK_VOLUME_MUTE: VKEY = VKEY(173u16);
pub const VK_VOLUME_UP: VKEY = VKEY(175u16);

pub const VK_W: VKEY = VKEY(87u16);
pub const VK_X: VKEY = VKEY(88u16);
pub const VK_XBUTTON1: VKEY = VKEY(5u16);
pub const VK_XBUTTON2: VKEY = VKEY(6u16);
pub const VK_Y: VKEY = VKEY(89u16);
pub const VK_Z: VKEY = VKEY(90u16);
pub const VK_ZOOM: VKEY = VKEY(251u16);
pub const VK_NONE: VKEY = VKEY(255u16);
