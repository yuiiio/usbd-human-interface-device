//! # HID USB Examples for usbd-hid-devices

#![no_std]

use adafruit_macropad::hal;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use embedded_graphics::mono_font::ascii::FONT_4X6;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_hal::digital::v2::InputPin;
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};
use hal::gpio::DynPin;
use hal::pac;
use hal::Spi;
use log::LevelFilter;
use sh1106::prelude::*;

pub mod logger;

pub static LOGGER: logger::Logger = logger::Logger;

type DisplaySpiInt = SpiInterface<Spi<hal::spi::Enabled, pac::SPI1, 8_u8>, DynPin, DynPin>;

static OLED_DISPLAY: Mutex<RefCell<Option<GraphicsMode<DisplaySpiInt>>>> =
    Mutex::new(RefCell::new(None));

pub const XTAL_FREQ_HZ: u32 = 12_000_000u32;

pub fn check_for_persisted_panic(restart: &dyn InputPin<Error = core::convert::Infallible>) {
    if let Some(msg) = panic_persist::get_panic_message_utf8() {
        cortex_m::interrupt::free(|cs| {
            let mut display_ref = OLED_DISPLAY.borrow(cs).borrow_mut();
            if let Some(display) = display_ref.as_mut() {
                draw_text_screen(display, msg).ok();
            }
        });

        while restart.is_high().unwrap() {
            cortex_m::asm::nop()
        }

        //USB boot with pin 13 for usb activity
        //Screen will continue to show panic message
        hal::rom_data::reset_to_usb_boot(0x1 << 13, 0x0);
    }
}

fn draw_text_screen<DI, E>(display: &mut GraphicsMode<DI>, text: &str) -> Result<(), E>
where
    DI: sh1106::interface::DisplayInterface<Error = E>,
{
    display.clear();
    let character_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .build();
    let bounds = Rectangle::new(Point::zero(), Size::new(128, 0));
    let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

    text_box.draw(display).unwrap();
    display.flush()?;

    Ok(())
}

pub fn init_display(spi: Spi<hal::spi::Enabled, pac::SPI1, 8_u8>, dc: DynPin, cs: DynPin) {
    let mut display: GraphicsMode<_> = sh1106::Builder::new().connect_spi(spi, dc, cs).into();

    display.init().unwrap();
    display.flush().unwrap();

    cortex_m::interrupt::free(|cs| {
        OLED_DISPLAY.borrow(cs).replace(Some(display));
        unsafe {
            log::set_logger_racy(&LOGGER)
                .map(|()| log::set_max_level(LevelFilter::Info))
                .unwrap();
        }
    });
}
