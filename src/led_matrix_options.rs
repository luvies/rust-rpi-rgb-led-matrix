use libc::{c_char, c_int, c_uint};
use std::error;
use std::ffi::CString;
use std::fmt;
use std::result;

pub type Result<E> = result::Result<(), E>;

////////////////////////////// Option Enums //////////////////////////////

pub enum ScanMode {
    Progressive = 0,
    Interlaced = 1,
}

pub enum RowAddressType {
    Direct = 0,
    ShiftRegister = 1,
    DirectABCDLine = 2,
    ABCShiftRegister = 3,
}

pub enum Multiplexing {
    Direct = 0,
    Stripe = 1,
    Checkered = 2,
    Spiral = 3,
    ZStripe = 4,
    ZnMirrorZStripe = 5,
    Coreman = 6,
}

////////////////////////////// Error Structs //////////////////////////////

/// The error type for the parallel count setter. This is returned if the value
/// for the parallel count is out of range.
#[derive(Debug, Clone)]
pub struct ParallelError {
    parallel: u16,
}

/// The error type for the PWM bits setter. This is returned if the value for
/// the PWM bits is out of range.
#[derive(Debug, Clone)]
pub struct PwmBitsError {
    pwm_bits: u8,
}

/// The error type for the brightness setter. This is returned if the value for
/// the brightness is out of range.
#[derive(Debug, Clone)]
pub struct BrightnessError {
    brightness: u8,
}

////////////////////////////// LED Matrix Options Impl //////////////////////////////

/// Parameters to create a new matrix.
#[repr(C, packed)]
pub struct Options {
    hardware_mapping: *mut c_char,
    rows: c_int,
    cols: c_int,
    chain_length: c_int,
    parallel: c_int,
    pwm_bits: c_int,
    pwm_lsb_nanoseconds: c_int,
    pwm_dither_bits: c_int,
    brightness: c_int,
    scan_mode: c_int,
    row_address_type: c_int,
    multiplexing: c_int,
    led_rgb_sequence: *mut c_char,
    pixel_mapper_config: *mut c_char,
    panel_type: *mut c_char,
    disable_hardware_pulsing: c_uint,
    show_refresh_rate: c_uint,
    inverse_colors: c_uint,
}

impl Options {
    /// Constructs a new LewMatrixOptions with the default values
    /// pre-configured.
    pub fn new() -> Options {
        Options {
            hardware_mapping: CString::new("regular").unwrap().into_raw(),
            rows: 32,
            cols: 32,
            chain_length: 1,
            parallel: 1,
            pwm_bits: 11,
            pwm_lsb_nanoseconds: 1000,
            pwm_dither_bits: 1,
            brightness: 100,
            scan_mode: ScanMode::Progressive as c_int,
            row_address_type: RowAddressType::Direct as c_int,
            multiplexing: Multiplexing::Direct as c_int,
            led_rgb_sequence: CString::new("RGB").unwrap().into_raw(),
            pixel_mapper_config: CString::new("").unwrap().into_raw(),
            panel_type: CString::new("").unwrap().into_raw(),
            disable_hardware_pulsing: 1,
            show_refresh_rate: 1,
            inverse_colors: 1,
        }
    }

    /// Name of the hardware mapping used. Can be an empty string.
    pub fn set_hardware_mapping(&mut self, mapping: &str) {
        unsafe {
            let _ = CString::from_raw(self.hardware_mapping);
            self.hardware_mapping = CString::new(mapping).unwrap().into_raw();
        }
    }

    /// The number of rows supported by the display. Typically 16, 32 or 64.
    pub fn set_rows(&mut self, rows: u16) {
        self.rows = rows as c_int;
    }

    /// The number of columns supported by the display. Typically 16, 32 or 64.
    pub fn set_cols(&mut self, cols: u16) {
        self.cols = cols as c_int;
    }

    /// The number of displays daisy-chained together.
    pub fn set_chain_length(&mut self, chain_length: u16) {
        self.chain_length = chain_length as c_int;
    }

    /// The number of parallel chains connected together. Old Pis with 26 pins
    /// only support 1, but newer Pis with 40 pins can support up to 3.
    pub fn set_parallel(&mut self, parallel: u16) -> Result<ParallelError> {
        if parallel < 1 || parallel > 3 {
            Err(ParallelError { parallel })
        } else {
            self.parallel = parallel as c_int;
            Ok(())
        }
    }

    /// Sets the PWM bits. Lower values decrease CPU and increase refresh-rate.
    pub fn set_pwm_bits(&mut self, pwm_bits: u8) -> Result<PwmBitsError> {
        if pwm_bits > 11 {
            Err(PwmBitsError { pwm_bits })
        } else {
            self.pwm_bits = pwm_bits as c_int;
            Ok(())
        }
    }

    /// The base time-unit for the on-time in the lowest significant bit in
    /// nanoseconds. Higher numbers provide higher quality (more accurate color,
    /// less ghosting), but have a negative impact on the frame rate.
    pub fn set_pwm_lsb_nanoseconds(&mut self, pwm_lsb_nanoseconds: u16) {
        self.pwm_lsb_nanoseconds = pwm_lsb_nanoseconds as c_int;
    }

    /// The lower bits that can be time-dithered for a higher refresh rate.
    pub fn set_pwm_dither_bits(&mut self, pwm_dither_bits: i32) {
        self.pwm_dither_bits = pwm_dither_bits as c_int;
    }

    /// The initial brightness of the panel in percent. Range of 1..100
    /// inclusive.
    pub fn set_brightness(&mut self, brightness: u8) -> Result<BrightnessError> {
        if brightness > 100 || brightness < 1 {
            Err(BrightnessError { brightness })
        } else {
            self.brightness = brightness as c_int;
            Ok(())
        }
    }

    /// The scan mode to use.
    pub fn set_scan_mode(&mut self, scan_mode: ScanMode) {
        self.scan_mode = scan_mode as c_int;
    }

    /// The row address type to use. Most panels will use direct, but some
    /// (typically some 64x64 panels) can use shift register.
    pub fn set_row_address_type(&mut self, row_address_type: RowAddressType) {
        self.row_address_type = row_address_type as c_int;
    }

    /// The type of multiplexing to use.
    pub fn set_multiplexing(&mut self, multiplexing: Multiplexing) {
        self.multiplexing = multiplexing as c_int;
    }

    /// The mapping of the RGB sequence. Most panels use the default RGB, but
    /// some may differ.
    pub fn set_led_rgb_sequence(&mut self, sequence: &str) {
        unsafe {
            let _ = CString::from_raw(self.led_rgb_sequence);
            self.led_rgb_sequence = CString::new(sequence).unwrap().into_raw();
        }
    }

    /// A string describing the sequence of pixel mappers that should be applied
    /// to the matrix. The string is a semicolon-separated list of pixel-mappers
    /// with an optional parameter.
    pub fn set_pixel_mapper_config(&mut self, pixel_mapper_config: &str) {
        unsafe {
            let _ = CString::from_raw(self.pixel_mapper_config);
            self.pixel_mapper_config = CString::new(pixel_mapper_config).unwrap().into_raw();
        }
    }

    /// The panel type. Normally just an empty string, but certain panels
    /// require an initialisation sequence.
    pub fn set_panel_type(&mut self, panel_type: &str) {
        unsafe {
            let _ = CString::from_raw(self.panel_type);
            self.panel_type = CString::new(panel_type).unwrap().into_raw();
        }
    }

    /// Allow the use of the hardware subsystem to create pulses. This won't do
    /// anything if output enable is not connected to GPIO 18.
    pub fn set_disable_hardware_pulsing(&mut self, disable: bool) {
        if disable {
            self.disable_hardware_pulsing = 1;
        } else {
            self.disable_hardware_pulsing = 0;
        }
    }

    pub fn set_show_refresh_rate(&mut self, show: bool) {
        if show {
            self.show_refresh_rate = 1;
        } else {
            self.show_refresh_rate = 0;
        }
    }

    /// Used if the matrix has inverse colours on.
    pub fn set_inverse_colors(&mut self, inverse: bool) {
        if inverse {
            self.inverse_colors = 1;
        } else {
            self.inverse_colors = 0;
        }
    }
}

impl Drop for Options {
    fn drop(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.hardware_mapping);
            let _ = CString::from_raw(self.led_rgb_sequence);
        }
    }
}

////////////////////////////// Error Struct Impls //////////////////////////////

impl fmt::Display for ParallelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "parallel count {} out of range (>= 1, <= 3)",
            self.parallel
        )
    }
}

impl error::Error for ParallelError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl fmt::Display for PwmBitsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PWM bits {} out of range (>= 0, <= 11)", self.pwm_bits)
    }
}

impl error::Error for PwmBitsError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl fmt::Display for BrightnessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "brightness {} out of range (>= 1, <= 100)",
            self.brightness
        )
    }
}

impl error::Error for BrightnessError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
