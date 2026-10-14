use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSURL};

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "BroxerAppDelegate"]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(application:openURLs:))]
        fn open_urls(&self, app: &NSApplication, urls: &NSArray<NSURL>) {
            for url in urls.iter() {
                #[allow(unused_unsafe)]
                if let Some(text) = unsafe { url.absoluteString() } {
                    crate::route_url(&text.to_string());
                }
            }
            #[allow(unused_unsafe)]
            unsafe {
                app.terminate(None)
            };
        }
    }
);

impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(());
        unsafe { msg_send![super(this), init] }
    }
}

pub fn run() {
    let mtm = MainThreadMarker::new().expect("router must start on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Prohibited);
    let delegate = AppDelegate::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
    // Launched without a URL (e.g. double-clicked): show settings instead of idling forever.
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(3));
        crate::open_settings();
        std::process::exit(0);
    });
    app.run();
}
