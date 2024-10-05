use bitflags::bitflags;
use bizarre_input_proc_macros::define_keys;

define_keys! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(u8)]
    pub Keyboard {
        (linux: 9) => Ecs,

        (linux: 24) => Q,
        (linux: 25) => W,
        (linux: 26) => E,
        (linux: 27) => R,
        (linux: 28) => T,
        (linux: 29) => Y,
        (linux: 30) => U,
        (linux: 31) => I,
        (linux: 32) => O,
        (linux: 33) => P,
        (linux: 38) => A,
        (linux: 39) => S,
        (linux: 40) => D,
        (linux: 41) => F,
        (linux: 42) => G,
        (linux: 43) => H,
        (linux: 44) => J,
        (linux: 45) => K,
        (linux: 46) => L,
        (linux: 52) => Z,
        (linux: 53) => X,
        (linux: 54) => C,
        (linux: 55) => V,
        (linux: 56) => B,
        (linux: 57) => N,
        (linux: 58) => M,


        (linux: 49) => Tilde,
        (linux: 10) => Digit1,
        (linux: 11) => Digit2,
        (linux: 12) => Digit3,
        (linux: 13) => Digit4,
        (linux: 14) => Digit5,
        (linux: 15) => Digit6,
        (linux: 16) => Digit7,
        (linux: 17) => Digit8,
        (linux: 18) => Digit9,
        (linux: 19) => Digit0,
        (linux: 20) => Minus,
        (linux: 21) => Equals,

        (linux: 22) => Backspace,
        (linux: 119) => Delete,
        (linux: 36) => Enter,
        (linux: 50) => LShift,
        (linux: 62) => RShift,
        (linux: 64) => LAlt,
        (linux: 108) => RAlt,
        (linux: 37) => LCtrl,
        (linux: 105) => RCtrl,
        (linux: 23) => Tab,
        (linux: 133) => Super,

        (linux: 59) => Comma,
        (linux: 60) => Dot,
        (linux: 61) => Slash,
        (linux: 51) => BackSlash,
        (linux: 34) => LBracket,
        (linux: 35) => RBracket,
        (linux: 47) => Colon,
        (linux: 48) => Quote,

        (linux: 77) => NumLock,
        (linux: 87) => Num1,
        (linux: 88) => Num2,
        (linux: 89) => Num3,
        (linux: 83) => Num4,
        (linux: 84) => Num5,
        (linux: 85) => Num6,
        (linux: 79) => Num7,
        (linux: 80) => Num8,
        (linux: 81) => Num9,
        (linux: 90) => Num0,

        (linux: 67) => F1,
        (linux: 68) => F2,
        (linux: 69) => F3,
        (linux: 70) => F4,
        (linux: 71) => F5,
        (linux: 72) => F6,
        (linux: 73) => F7,
        (linux: 74) => F8,
        (linux: 75) => F9,
        (linux: 76) => F10,
        (linux: 95) => F11,
        (linux: 96) => F12,
    }
}

bitflags! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct KeyboardModifier: u8 {
                const None      = 0;

                const LShift    = 0b_0000_0001;
                const RShift    = 0b_0000_0010;
                const Shift     = Self::LShift.bits() | Self::RShift.bits();

                const LAlt      = 0b_0000_0100;
                const RAlt      = 0b_0000_1000;
                const Alt       = Self::LAlt.bits() | Self::RAlt.bits();

                const LCtrl     = 0b_0001_0000;
                const RCtrl     = 0b_0010_0000;
                const Ctrl      = Self::LCtrl.bits() | Self::RCtrl.bits();

                const Super     = 0b_1100_0000;

                const _ = !0;
        }
}

impl From<Keyboard> for KeyboardModifier {
    fn from(value: Keyboard) -> Self {
        match value {
            Keyboard::LShift => KeyboardModifier::LShift,
            Keyboard::RShift => KeyboardModifier::RShift,
            Keyboard::LAlt => KeyboardModifier::LAlt,
            Keyboard::RAlt => KeyboardModifier::RAlt,
            Keyboard::LCtrl => KeyboardModifier::LCtrl,
            Keyboard::RCtrl => KeyboardModifier::RCtrl,
            Keyboard::Super => KeyboardModifier::Super,
            _ => KeyboardModifier::None,
        }
    }
}

impl Default for KeyboardModifier {
    fn default() -> Self {
        Self::None
    }
}
