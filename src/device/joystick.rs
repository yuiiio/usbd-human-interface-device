//!HID joystick
use crate::usb_class::prelude::*;
use core::default::Default;
use fugit::ExtU32;
use packed_struct::prelude::*;
use usb_device::bus::UsbBus;
use usb_device::class_prelude::UsbBusAllocator;

#[rustfmt::skip]
pub const JOYSTICK_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)         5,   1
    0x09, 0x04, // Usage (Joystick)                     9,   4
    0xa1, 0x01, // Collection (Application)             161, 1
    0x09, 0x01, //   Usage Page (Pointer)               9,   1
    0xa1, 0x00, //   Collection (Physical)              161, 0
    0x09, 0x30, //     Usage (LX)                        9,   48
    0x09, 0x31, //     Usage (LY)                        9,   49
    0x09, 0x33, //     Usage (RX)                        9,   51
    0x09, 0x34, //     Usage (RY)                        9,   52
    0x15, 0x00, //     Logical Minimum (0)              21,  0
    0x25, 0xff, //     Logical Maximum (255)            37,  255
    0x75, 0x08, //     Report Size (8)                  117, 8
    0x95, 0x04, //     Report count (4)                 149, 4,
    0x81, 0x02, //     Input (Data, Variable, Absolute) 129, 2,
    0xc0,       //   End Collection                     192,
    0xa1, 0x00, //   Collection (Physical)              161, 0
    0x09, 0x32, //     Usage (LZ)                        9,   50
    0x09, 0x35, //     Usage (RZ)                        9,   53
    0x15, 0x00, //     Logical Minimum (0)              21,  0
    0x25, 0xff, //     Logical Maximum (255)            37,  255
    0x75, 0x08, //     Report Size (8)                  117, 8
    0x95, 0x02, //     Report count (2)                 149, 2,
    0x81, 0x02, //     Input (Data, Variable, Absolute) 129, 2,
    0xc0,       //   End Collection                     192,
    0x05, 0x09, //   Usage Page (Button)                5,   9,
    0x19, 0x01, //   Usage Minimum (0)                  25,  1,
    0x29, 0x08, //   Usage Maximum (8)                  41,  8,
    0x15, 0x00, //   Logical Minimum (0)                21,  0
    0x25, 0x01, //   Logical Maximum (1)                37,  1,
    0x75, 0x01, //   Report Size (1)                    117, 1,
    0x95, 0x08, //   Report Count (8)                   149, 8
    0x81, 0x02, //   Input (Data, Variable, Absolute)   129, 2,
    
	0x05, 0x01,							/*   USAGE_PAGE (Generic Desktop) */
	0x09, 0x39,							/*   USAGE (Hat switch) */
	0x09, 0x39,							/*   USAGE (Hat switch) */
	0x15, 0x01,							/*   LOGICAL_MINIMUM (1) */
	0x25, 0x08,							/*   LOGICAL_MAXIMUM (8) */
	0x95, 0x02,							/*   REPORT_COUNT (2) */
	0x75, 0x04,							/*   REPORT_SIZE (4) */
	0x81, 0x02,							/*   INPUT (Data,Var,Abs) */

    0xc0,       // End Collection                       192
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default, PackedStruct)]
#[packed_struct(endian = "lsb", size_bytes = "8")]
pub struct JoystickReport {
    #[packed_field]
    pub lx: u8,
    #[packed_field]
    pub ly: u8,
    #[packed_field]
    pub rx: u8,
    #[packed_field]
    pub ry: u8,
    #[packed_field]
    pub lz: u8,
    #[packed_field]
    pub rz: u8,
    #[packed_field]
    pub buttons: u8,
    #[packed_field]
    pub hat_switch: u8,
}

pub struct Joystick<'a, B: UsbBus> {
    interface: Interface<'a, B, InBytes8, OutNone, ReportSingle>,
}

impl<'a, B: UsbBus> Joystick<'a, B> {
    pub fn write_report(&mut self, report: &JoystickReport) -> Result<(), UsbHidError> {
        let data = report.pack().map_err(|_| {
            error!("Error packing JoystickReport");
            UsbHidError::SerializationError
        })?;
        self.interface
            .write_report(&data)
            .map(|_| ())
            .map_err(UsbHidError::from)
    }
}

impl<'a, B: UsbBus> DeviceClass<'a> for Joystick<'a, B> {
    type I = Interface<'a, B, InBytes8, OutNone, ReportSingle>;

    fn interface(&mut self) -> &mut Self::I {
        &mut self.interface
    }

    fn reset(&mut self) {}

    fn tick(&mut self) -> Result<(), UsbHidError> {
        Ok(())
    }
}

pub struct JoystickConfig<'a> {
    interface: InterfaceConfig<'a, InBytes8, OutNone, ReportSingle>,
}

impl<'a> Default for JoystickConfig<'a> {
    #[must_use]
    fn default() -> Self {
        Self::new(
            unwrap!(unwrap!(InterfaceBuilder::new(JOYSTICK_DESCRIPTOR))
                .boot_device(InterfaceProtocol::None)
                .description("Joystick")
                .in_endpoint(10.millis()))
            .without_out_endpoint()
            .build(),
        )
    }
}

impl<'a> JoystickConfig<'a> {
    #[must_use]
    pub fn new(interface: InterfaceConfig<'a, InBytes8, OutNone, ReportSingle>) -> Self {
        Self { interface }
    }
}

impl<'a, B: UsbBus + 'a> UsbAllocatable<'a, B> for JoystickConfig<'a> {
    type Allocated = Joystick<'a, B>;

    fn allocate(self, usb_alloc: &'a UsbBusAllocator<B>) -> Self::Allocated {
        Self::Allocated {
            interface: Interface::new(usb_alloc, self.interface),
        }
    }
}
