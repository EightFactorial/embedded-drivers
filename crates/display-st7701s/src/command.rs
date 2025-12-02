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
    ST7701S_READ_ID_1: 0x04,
    ST7701S_READ_ID_2: 0x05,
    ST7701S_READ_ID_3: 0x06,
    ST7701S_SLEEP_ENTER: 0x10,
    ST7701S_SLEEP_EXIT: 0x11,
    ST7701S_PARTIAL_MODE: 0x12,
    ST7701S_NORMAL_MODE: 0x13,
    ST7701S_INVERSION_OFF: 0x20,
    ST7701S_INVERSION_ON: 0x21,
    ST7701S_ALL_PIXEL_OFF: 0x22,
    ST7701S_ALL_PIXEL_ON: 0x23,
    ST7701S_DISPLAY_OFF: 0x28,
    ST7701S_DISPLAY_ON: 0x29,
    ST7701S_SET_COLUMN_ADDR: 0x2A,
    ST7701S_SET_PAGE_ADDR: 0x2B,
    ST7701S_MEMORY_WRITE: 0x2C,
    ST7701S_MEMORY_READ: 0x2E,
    ST7701S_SET_ADDRESS_MODE: 0x36,
    ST7701S_IDLE_OFF: 0x38,
    ST7701S_IDLE_ON: 0x39,
    ST7701S_PIXEL_FORMAT: 0x3A,
    ST7701S_DISPLAY_BRIGHTNESS: 0x51,
    // Command2 BK0
    ST7701S_COLOR_CONTROL: 0xCD,

    ST7701S_CMD_BANK_SELECT: 0xFF,
}
