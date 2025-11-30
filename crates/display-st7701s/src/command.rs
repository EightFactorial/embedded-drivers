//! All documented commands from the ST7701S datasheet.
#![expect(dead_code, reason = "Not all register values are used")]

macro_rules! command {
    ($($ident:ident: $addr:expr,)+) => {
        $(pub(super) const $ident: u8 = $addr;)+
    };
}

command! {
    ST7701S_NOP: 0x00,
    ST7701S_SOFT_RESET: 0x01,
    ST7701S_SLEEP_ENTER: 0x10,
    ST7701S_SLEEP_EXIT: 0x11,
    ST7701S_PARTIAL_MODE: 0x12,
    ST7701S_NORMAL_MODE: 0x13,
    ST7701S_DISPLAY_OFF: 0x28,
    ST7701S_DISPLAY_ON: 0x29,
    ST7701S_IDLE_OFF: 0x38,
    ST7701S_IDLE_ON: 0x39,
}
