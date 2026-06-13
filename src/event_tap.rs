use std::ffi::c_void;
use std::ptr;

use crate::config::Config;

type Boolean = u8;
type CFIndex = isize;
type CFAllocatorRef = *const c_void;
type CFRunLoopRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFStringRef = *const c_void;
type CFDictionaryRef = *const c_void;
type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CFMachPortRef = *mut c_void;
type CGEventType = u32;
type CGEventField = u32;

const SCROLL_WHEEL: CGEventType = 22;
const TAP_DISABLED_BY_TIMEOUT: CGEventType = 0xffff_fffe;
const TAP_DISABLED_BY_USER_INPUT: CGEventType = 0xffff_ffff;
const DELTA_AXIS_1: CGEventField = 11;
const DELTA_AXIS_2: CGEventField = 12;
const IS_CONTINUOUS: CGEventField = 88;
const FIXED_PT_DELTA_AXIS_1: CGEventField = 93;
const FIXED_PT_DELTA_AXIS_2: CGEventField = 94;
const POINT_DELTA_AXIS_1: CGEventField = 96;
const POINT_DELTA_AXIS_2: CGEventField = 97;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> Boolean;
    static kAXTrustedCheckOptionPrompt: CFStringRef;
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: extern "C" fn(
            CGEventTapProxy,
            CGEventType,
            CGEventRef,
            *mut c_void,
        ) -> CGEventRef,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: Boolean);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: CGEventField, value: i64);
    fn CGEventGetDoubleValueField(event: CGEventRef, field: CGEventField) -> f64;
    fn CGEventSetDoubleValueField(event: CGEventRef, field: CGEventField, value: f64);
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFBooleanTrue: *const c_void;
    static kCFRunLoopCommonModes: CFStringRef;
    fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: CFIndex,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CFDictionaryRef;
    fn CFRelease(value: *const c_void);
    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(run_loop: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRun();
}

pub fn run() -> Result<(), String> {
    let config = Config::load_or_create()?;
    if !config.enabled {
        println!("ScrollSplit is disabled in the configuration");
        return Ok(());
    }
    if !request_accessibility_permission() {
        return Err(
            "Accessibility permission is required. Enable ScrollSplit in System Settings > Privacy & Security > Accessibility, then run it again."
                .to_owned(),
        );
    }

    let config = Box::into_raw(Box::new(config));
    let tap = unsafe {
        CGEventTapCreate(
            0,
            0,
            0,
            1_u64 << SCROLL_WHEEL,
            event_callback,
            config.cast(),
        )
    };
    if tap.is_null() {
        unsafe {
            drop(Box::from_raw(config));
        }
        return Err("cannot create CGEventTap; verify Accessibility permission".to_owned());
    }

    let source = unsafe { CFMachPortCreateRunLoopSource(ptr::null(), tap, 0) };
    if source.is_null() {
        unsafe {
            CFRelease(tap);
            drop(Box::from_raw(config));
        }
        return Err("cannot create run loop source".to_owned());
    }

    unsafe {
        CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, 1);
    }
    println!("ScrollSplit is running");
    unsafe {
        CFRunLoopRun();
        CFRelease(source);
        CFRelease(tap);
        drop(Box::from_raw(config));
    }
    Ok(())
}

fn request_accessibility_permission() -> bool {
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];
        let options = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            ptr::null(),
            ptr::null(),
        );
        let trusted = AXIsProcessTrustedWithOptions(options) != 0;
        CFRelease(options);
        trusted
    }
}

extern "C" fn event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    if event_type == TAP_DISABLED_BY_TIMEOUT || event_type == TAP_DISABLED_BY_USER_INPUT {
        return event;
    }
    if event_type != SCROLL_WHEEL || event.is_null() || user_info.is_null() {
        return event;
    }

    let config = unsafe { &*user_info.cast::<Config>() };
    let continuous = unsafe { CGEventGetIntegerValueField(event, IS_CONTINUOUS) != 0 };
    let reverse_vertical = if continuous {
        config.reverse_trackpad
    } else {
        config.reverse_mouse
    };

    if reverse_vertical {
        invert_axis(
            event,
            DELTA_AXIS_1,
            FIXED_PT_DELTA_AXIS_1,
            POINT_DELTA_AXIS_1,
        );
    }
    if config.reverse_horizontal {
        invert_axis(
            event,
            DELTA_AXIS_2,
            FIXED_PT_DELTA_AXIS_2,
            POINT_DELTA_AXIS_2,
        );
    }
    event
}

fn invert_axis(event: CGEventRef, delta: CGEventField, fixed: CGEventField, point: CGEventField) {
    unsafe {
        let delta_value = CGEventGetIntegerValueField(event, delta);
        let point_value = CGEventGetIntegerValueField(event, point);
        let fixed_value = CGEventGetDoubleValueField(event, fixed);
        CGEventSetIntegerValueField(event, delta, -delta_value);
        CGEventSetIntegerValueField(event, point, -point_value);
        CGEventSetDoubleValueField(event, fixed, -fixed_value);
    }
}
