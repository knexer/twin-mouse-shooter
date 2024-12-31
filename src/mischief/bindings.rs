/* automatically generated by rust-bindgen 0.68.1 */

pub const MANYMOUSE_VERSION: &[u8; 6] = b"0.0.3\0";
pub const ManyMouseEventType_MANYMOUSE_EVENT_ABSMOTION: ManyMouseEventType = 0;
pub const ManyMouseEventType_MANYMOUSE_EVENT_RELMOTION: ManyMouseEventType = 1;
pub const ManyMouseEventType_MANYMOUSE_EVENT_BUTTON: ManyMouseEventType = 2;
pub const ManyMouseEventType_MANYMOUSE_EVENT_SCROLL: ManyMouseEventType = 3;
pub const ManyMouseEventType_MANYMOUSE_EVENT_DISCONNECT: ManyMouseEventType = 4;
pub const ManyMouseEventType_MANYMOUSE_EVENT_MAX: ManyMouseEventType = 5;
pub type ManyMouseEventType = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ManyMouseEvent {
    pub type_: ManyMouseEventType,
    pub device: ::std::os::raw::c_uint,
    pub item: ::std::os::raw::c_uint,
    pub value: ::std::os::raw::c_int,
    pub minval: ::std::os::raw::c_int,
    pub maxval: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_ManyMouseEvent() {
    const UNINIT: ::std::mem::MaybeUninit<ManyMouseEvent> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<ManyMouseEvent>(),
        24usize,
        concat!("Size of: ", stringify!(ManyMouseEvent))
    );
    assert_eq!(
        ::std::mem::align_of::<ManyMouseEvent>(),
        4usize,
        concat!("Alignment of ", stringify!(ManyMouseEvent))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).type_) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseEvent),
            "::",
            stringify!(type_)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).device) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseEvent),
            "::",
            stringify!(device)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).item) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseEvent),
            "::",
            stringify!(item)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).value) as usize - ptr as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseEvent),
            "::",
            stringify!(value)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).minval) as usize - ptr as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseEvent),
            "::",
            stringify!(minval)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).maxval) as usize - ptr as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseEvent),
            "::",
            stringify!(maxval)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ManyMouseDriver {
    pub driver_name: *const ::std::os::raw::c_char,
    pub init: ::std::option::Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub quit: ::std::option::Option<unsafe extern "C" fn()>,
    pub name: ::std::option::Option<
        unsafe extern "C" fn(index: ::std::os::raw::c_uint) -> *const ::std::os::raw::c_char,
    >,
    pub poll: ::std::option::Option<
        unsafe extern "C" fn(event: *mut ManyMouseEvent) -> ::std::os::raw::c_int,
    >,
}
#[test]
fn bindgen_test_layout_ManyMouseDriver() {
    const UNINIT: ::std::mem::MaybeUninit<ManyMouseDriver> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<ManyMouseDriver>(),
        40usize,
        concat!("Size of: ", stringify!(ManyMouseDriver))
    );
    assert_eq!(
        ::std::mem::align_of::<ManyMouseDriver>(),
        8usize,
        concat!("Alignment of ", stringify!(ManyMouseDriver))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).driver_name) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseDriver),
            "::",
            stringify!(driver_name)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).init) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseDriver),
            "::",
            stringify!(init)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).quit) as usize - ptr as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseDriver),
            "::",
            stringify!(quit)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).name) as usize - ptr as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseDriver),
            "::",
            stringify!(name)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).poll) as usize - ptr as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(ManyMouseDriver),
            "::",
            stringify!(poll)
        )
    );
}
extern "C" {
    pub fn ManyMouse_Init() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ManyMouse_DriverName() -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn ManyMouse_Quit();
}
extern "C" {
    pub fn ManyMouse_DeviceName(index: ::std::os::raw::c_uint) -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn ManyMouse_PollEvent(event: *mut ManyMouseEvent) -> ::std::os::raw::c_int;
}
