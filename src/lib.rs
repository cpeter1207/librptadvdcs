//! Native 48 kHz DCS receive and transmit behind a versioned C descriptor.

#![deny(warnings)]
#![cfg_attr(coverage, feature(coverage_attribute))]

use std::{ffi::c_char, mem::size_of, ptr};

mod dcs_receive;
mod dcs_transmit;

const NATIVE_SAMPLE_RATE_HZ: u32 = 48_000;
const ABI_VERSION: u32 = 1;
const CAPABILITY_NAME: &[u8] = b"rptadv.dcs\0";
const OK: i32 = 0;
const INVALID_ARGUMENT: i32 = -1;

/// Owned state for one DCS receive stream.
pub struct DcsReceive(dcs_receive::ReceiveState);

/// Owned state for one DCS transmit stream or prepared configuration.
#[derive(Default)]
pub struct DcsTransmit(dcs_transmit::TransmitState);

/// Immutable ABI 1 DCS function table.
#[repr(C)]
pub struct DcsDescriptor {
    struct_size: u32,
    abi_version: u32,
    capability_name: *const c_char,
    code_supported: extern "C" fn(i32) -> u32,
    configured_code: extern "C" fn(i32) -> i32,
    receive_create: unsafe extern "C" fn(*mut *mut DcsReceive) -> i32,
    receive_destroy: unsafe extern "C" fn(*mut DcsReceive),
    receive_configure: unsafe extern "C" fn(*mut DcsReceive, i32, u32) -> i32,
    receive_valid: unsafe extern "C" fn(*const DcsReceive) -> u32,
    receive_enabled: unsafe extern "C" fn(*const DcsReceive) -> u32,
    receive_process: unsafe extern "C" fn(*mut DcsReceive, *const f32, u32, *mut u32) -> i32,
    transmit_create: unsafe extern "C" fn(*mut *mut DcsTransmit) -> i32,
    transmit_destroy: unsafe extern "C" fn(*mut DcsTransmit),
    transmit_configure: unsafe extern "C" fn(*mut DcsTransmit, i32, u32) -> i32,
    transmit_update_config_from: unsafe extern "C" fn(*mut DcsTransmit, *const DcsTransmit) -> i32,
    transmit_generate: unsafe extern "C" fn(*mut DcsTransmit, *mut f32, u32, f32, u32, u32) -> i32,
}

// SAFETY: the descriptor and every function address in it are immutable.
unsafe impl Sync for DcsDescriptor {}

static DESCRIPTOR: DcsDescriptor = DcsDescriptor {
    struct_size: size_of::<DcsDescriptor>() as u32,
    abi_version: ABI_VERSION,
    capability_name: CAPABILITY_NAME.as_ptr().cast::<c_char>(),
    code_supported,
    configured_code,
    receive_create,
    receive_destroy,
    receive_configure,
    receive_valid,
    receive_enabled,
    receive_process,
    transmit_create,
    transmit_destroy,
    transmit_configure,
    transmit_update_config_from,
    transmit_generate,
};

/// Return the immutable ABI 1 descriptor; callers validate all three prefix fields.
#[unsafe(no_mangle)]
pub extern "C" fn rptadv_dcs_descriptor() -> *const DcsDescriptor {
    &DESCRIPTOR
}

extern "C" fn code_supported(code: i32) -> u32 {
    u32::from(dcs_receive::code_supported(code))
}

extern "C" fn configured_code(code: i32) -> i32 {
    dcs_transmit::configured_code(code)
}

unsafe extern "C" fn receive_create(out: *mut *mut DcsReceive) -> i32 {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return INVALID_ARGUMENT;
    };
    *out = Box::into_raw(Box::new(DcsReceive(dcs_receive::ReceiveState::default())));
    OK
}

unsafe extern "C" fn receive_destroy(handle: *mut DcsReceive) {
    if !handle.is_null() {
        // SAFETY: the owner calls this once after callback use has stopped.
        drop(unsafe { Box::from_raw(handle) });
    }
}

unsafe extern "C" fn receive_configure(handle: *mut DcsReceive, code: i32, inverted: u32) -> i32 {
    let Some(handle) = (unsafe { handle.as_mut() }) else {
        return INVALID_ARGUMENT;
    };
    handle.0.configure(code, inverted);
    OK
}

unsafe extern "C" fn receive_valid(handle: *const DcsReceive) -> u32 {
    u32::from(unsafe { handle.as_ref() }.is_some_and(|handle| handle.0.valid()))
}

unsafe extern "C" fn receive_enabled(handle: *const DcsReceive) -> u32 {
    u32::from(unsafe { handle.as_ref() }.is_some_and(|handle| handle.0.enabled()))
}

unsafe extern "C" fn receive_process(
    handle: *mut DcsReceive,
    stereo: *const f32,
    frames: u32,
    valid_out: *mut u32,
) -> i32 {
    if handle.is_null() || valid_out.is_null() || (frames != 0 && stereo.is_null()) {
        return INVALID_ARGUMENT;
    }
    // SAFETY: the caller owns the handle exclusively and supplies 2*frames readable samples.
    let valid = unsafe { (*handle).0.process(stereo, frames) };
    unsafe { valid_out.write(u32::from(valid)) };
    OK
}

unsafe extern "C" fn transmit_create(out: *mut *mut DcsTransmit) -> i32 {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return INVALID_ARGUMENT;
    };
    *out = Box::into_raw(Box::new(DcsTransmit::default()));
    OK
}

unsafe extern "C" fn transmit_destroy(handle: *mut DcsTransmit) {
    if !handle.is_null() {
        // SAFETY: the owner calls this once after callback use has stopped.
        drop(unsafe { Box::from_raw(handle) });
    }
}

unsafe extern "C" fn transmit_configure(handle: *mut DcsTransmit, code: i32, inverted: u32) -> i32 {
    let Some(handle) = (unsafe { handle.as_mut() }) else {
        return INVALID_ARGUMENT;
    };
    handle.0.configure(code, inverted);
    OK
}

unsafe extern "C" fn transmit_update_config_from(
    handle: *mut DcsTransmit,
    prepared: *const DcsTransmit,
) -> i32 {
    if handle.is_null() || prepared.is_null() || ptr::eq(handle, prepared.cast_mut()) {
        return INVALID_ARGUMENT;
    }
    // SAFETY: caller owns distinct handles, with the prepared handle held immutably.
    unsafe { (*handle).0.update_config_from(&(*prepared).0) };
    OK
}

unsafe extern "C" fn transmit_generate(
    handle: *mut DcsTransmit,
    output: *mut f32,
    frames: u32,
    peak: f32,
    enabled: u32,
    turnoff: u32,
) -> i32 {
    if handle.is_null() || (frames != 0 && output.is_null()) {
        return INVALID_ARGUMENT;
    }
    // SAFETY: caller owns the handle exclusively and supplies frames writable samples.
    unsafe {
        dcs_transmit::generate(
            &mut (*handle).0,
            output,
            frames,
            dcs_transmit::GenerateConfig {
                peak,
                enabled,
                turnoff,
            },
        )
    };
    OK
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn descriptor_lifecycle_buffer_contract_and_prepared_phase() {
        let api = unsafe { &*rptadv_dcs_descriptor() };
        assert_eq!(rptadv_dcs_descriptor(), &raw const DESCRIPTOR);
        assert_eq!(api.struct_size as usize, size_of::<DcsDescriptor>());
        assert_eq!(api.abi_version, ABI_VERSION);
        assert_eq!(
            unsafe { CStr::from_ptr(api.capability_name) }.to_bytes(),
            b"rptadv.dcs"
        );
        assert_eq!((api.code_supported)(0o023), 1);
        assert_eq!((api.code_supported)(-1), 0);
        assert_eq!((api.configured_code)(0o777), 0o777);
        assert_eq!((api.configured_code)(0o1000), -1);

        assert_eq!(
            unsafe { (api.receive_create)(ptr::null_mut()) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.transmit_create)(ptr::null_mut()) },
            INVALID_ARGUMENT
        );
        let mut rx = ptr::null_mut();
        let mut whole = ptr::null_mut();
        let mut split = ptr::null_mut();
        let mut prepared = ptr::null_mut();
        assert_eq!(unsafe { (api.receive_create)(&mut rx) }, OK);
        for out in [&mut whole, &mut split, &mut prepared] {
            assert_eq!(unsafe { (api.transmit_create)(out) }, OK);
        }
        assert!(!rx.is_null() && !whole.is_null() && !split.is_null() && !prepared.is_null());
        assert_eq!(unsafe { (api.receive_valid)(rx) }, 0);
        assert_eq!(unsafe { (api.receive_enabled)(rx) }, 0);
        assert_eq!(unsafe { (api.receive_valid)(ptr::null()) }, 0);
        assert_eq!(unsafe { (api.receive_enabled)(ptr::null()) }, 0);
        assert_eq!(
            unsafe { (api.receive_configure)(ptr::null_mut(), 0o023, 0) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.transmit_configure)(ptr::null_mut(), 0o023, 0) },
            INVALID_ARGUMENT
        );

        assert_eq!(unsafe { (api.receive_configure)(rx, 0o023, 0) }, OK);
        assert_eq!(unsafe { (api.receive_enabled)(rx) }, 1);
        for tx in [whole, split, prepared] {
            assert_eq!(unsafe { (api.transmit_configure)(tx, 0o023, 0) }, OK);
        }
        let mut valid = 0;
        assert_eq!(
            unsafe { (api.receive_process)(ptr::null_mut(), ptr::null(), 0, &mut valid) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.receive_process)(rx, ptr::null(), 0, ptr::null_mut()) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.receive_process)(rx, ptr::null(), 1, &mut valid) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.receive_process)(rx, ptr::null(), 0, &mut valid) },
            OK
        );

        let mut stereo = vec![0.0; 48_000 * 2];
        let word = 0x76_3813_u32;
        for (index, frame) in stereo.chunks_exact_mut(2).enumerate() {
            let bit = (index as u64 * 1_344 / 480_000) % 23;
            frame[0] = if word & (1 << bit) != 0 {
                0.375
            } else {
                -0.375
            };
            frame[1] = -frame[0];
        }
        for chunk in stereo.chunks(514) {
            assert_eq!(
                unsafe {
                    (api.receive_process)(rx, chunk.as_ptr(), (chunk.len() / 2) as u32, &mut valid)
                },
                OK
            );
        }
        assert_eq!(valid, 1);
        assert_eq!(unsafe { (api.receive_valid)(rx) }, 1);

        let mut expected = [0.0; 9_217];
        let mut actual = [0.0; 9_217];
        assert_eq!(
            unsafe { (api.transmit_generate)(whole, expected.as_mut_ptr(), 9_217, 0.25, 1, 0) },
            OK
        );
        for chunk in actual.chunks_mut(791) {
            assert_eq!(
                unsafe {
                    (api.transmit_generate)(
                        split,
                        chunk.as_mut_ptr(),
                        chunk.len() as u32,
                        0.25,
                        1,
                        0,
                    )
                },
                OK
            );
        }
        assert_eq!(actual, expected);
        assert_eq!(
            unsafe { (api.transmit_update_config_from)(split, prepared) },
            OK
        );
        assert_eq!(unsafe { (api.receive_valid)(rx) }, 1);
        assert_eq!(
            unsafe { (api.transmit_generate)(whole, expected.as_mut_ptr(), 9_217, 0.25, 1, 0) },
            OK
        );
        assert_eq!(
            unsafe { (api.transmit_generate)(split, actual.as_mut_ptr(), 9_217, 0.25, 1, 0) },
            OK
        );
        assert_eq!(actual, expected);
        assert_eq!(
            unsafe { (api.transmit_generate)(split, actual.as_mut_ptr(), 9_217, 0.25, 1, 1) },
            OK
        );
        assert_eq!(
            unsafe { (api.transmit_generate)(split, ptr::null_mut(), 0, 0.25, 1, 0) },
            OK
        );
        assert_eq!(
            unsafe { (api.transmit_generate)(split, actual.as_mut_ptr(), 9_217, 0.0, 1, 0) },
            OK
        );
        assert!(actual.iter().all(|sample| sample.to_bits() == 0));
        assert_eq!(
            unsafe { (api.transmit_generate)(ptr::null_mut(), actual.as_mut_ptr(), 1, 0.25, 1, 0) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.transmit_generate)(split, ptr::null_mut(), 1, 0.25, 1, 0) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.transmit_update_config_from)(ptr::null_mut(), prepared) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.transmit_update_config_from)(split, ptr::null()) },
            INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe { (api.transmit_update_config_from)(split, split) },
            INVALID_ARGUMENT
        );
        assert_eq!(unsafe { (api.receive_configure)(rx, -1, 0) }, OK);
        assert_eq!(unsafe { (api.receive_enabled)(rx) }, 0);
        assert_eq!(unsafe { (api.receive_valid)(rx) }, 0);

        unsafe { (api.receive_destroy)(rx) };
        for tx in [whole, split, prepared] {
            unsafe { (api.transmit_destroy)(tx) };
        }
        unsafe { (api.receive_destroy)(ptr::null_mut()) };
        unsafe { (api.transmit_destroy)(ptr::null_mut()) };
    }
}
