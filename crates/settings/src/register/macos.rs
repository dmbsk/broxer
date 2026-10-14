use std::io;
use std::path::Path;

use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::{CFURL, CFURLRef};

pub const AFTER_REGISTER: &str =
    "Registered. Confirm the macOS prompt to make Broxer your default browser.";
const ROUTER_BUNDLE_ID: &str = "dev.dmbsk.broxer";

#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSRegisterURL(url: CFURLRef, update: u8) -> i32;
    fn LSSetDefaultHandlerForURLScheme(scheme: CFStringRef, handler: CFStringRef) -> i32;
}

pub fn register(router_app: &Path) -> io::Result<()> {
    let url = CFURL::from_path(router_app, true)
        .ok_or_else(|| io::Error::other("invalid Broxer.app path"))?;
    check("LSRegisterURL", unsafe { LSRegisterURL(url.as_concrete_TypeRef(), 1) })?;
    let handler = CFString::new(ROUTER_BUNDLE_ID);
    for scheme in ["http", "https"] {
        let scheme = CFString::new(scheme);
        check("LSSetDefaultHandlerForURLScheme", unsafe {
            LSSetDefaultHandlerForURLScheme(scheme.as_concrete_TypeRef(), handler.as_concrete_TypeRef())
        })?;
    }
    Ok(())
}

pub fn unregister() -> io::Result<()> {
    Err(io::Error::other(
        "macOS cannot unset a default browser; pick another one in System Settings → Desktop & Dock → Default web browser",
    ))
}

fn check(call: &str, status: i32) -> io::Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(io::Error::other(format!("{call} failed with status {status}")))
    }
}
