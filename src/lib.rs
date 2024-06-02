use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn set_city_name(c_city_name: *const c_char) {
    // for NULL-terminated C strings, it's a little bit clumsier
    let city_name = unsafe { CStr::from_ptr(c_city_name).to_string_lossy().into_owned() };

    println!("city_name = <{:?}>", city_name);
}

#[no_mangle]
pub extern "C" fn set_city_geometry(lat: f64, lng: f64) {
    println!("Latitude = <{}> Longtude = {}", lat, lng);
}

#[no_mangle]
pub extern "C" fn is_city_valid() -> i32 {
    return 100;
}

#[no_mangle]
pub extern "C" fn get_city_temperature() -> f32 {
    return 30.5;
}
