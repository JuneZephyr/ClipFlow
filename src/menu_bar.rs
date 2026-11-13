//! The little ClipFlow icon that lives in the menu bar at the top of the
//! screen.
//!
//! A clipboard manager is happiest up there: it stays out of the way until it
//! is asked for, instead of keeping a window open below everything else. The
//! icon's menu shows or hides the window and quits the app.
//!
//! The window is moved with AppKit itself rather than through an egui viewport
//! command. A hidden window gets no frames at all, so an egui command sent
//! while it is hidden would only be acted on whenever something else happened
//! to wake the app up — and then only if a frame followed. Asking the window
//! directly works the moment the menu item is chosen, and macOS tells winit
//! about the window becoming key again, so the app repaints as usual.
//!
//! This file is only compiled on macOS; the app runs unchanged elsewhere.

use std::sync::atomic::{AtomicPtr, Ordering};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{declare_class, extern_methods, mutability, msg_send, sel, ClassType, DeclaredClass};
use objc2_app_kit::{NSApplication, NSImage, NSMenu, NSStatusBar};
use objc2_foundation::{MainThreadMarker, NSData, NSObject, NSSize, NSString, NSTimer};

declare_class!(
    struct MenuTarget;

    unsafe impl ClassType for MenuTarget {
        type Super = NSObject;
        type Mutability = mutability::Mutable;
        const NAME: &'static str = "ClipFlowMenuTarget";
    }

    impl DeclaredClass for MenuTarget {}

    unsafe impl MenuTarget {
        /// Show the window, or hide it if it is already showing.
        #[method(toggleWindow:)]
        fn toggle_window(&self, _sender: &AnyObject) {
            let mtm = match MainThreadMarker::new() {
                Some(m) => m,
                None => return,
            };
            let Some(window) = window() else {
                return;
            };
            let visible: bool = unsafe { msg_send![window, isVisible] };
            if visible {
                unsafe {
                    let _: () = msg_send![window, orderOut: std::ptr::null_mut::<AnyObject>()];
                }
            } else {
                // Ordering the window back in is not enough on its own: the app
                // has to come forward too, otherwise the window appears behind
                // whatever the person is working in.
                let app = NSApplication::sharedApplication(mtm);
                #[allow(deprecated)]
                app.activateIgnoringOtherApps(true);
                unsafe {
                    app.activate();
                    let _: () =
                        msg_send![window, makeKeyAndOrderFront: std::ptr::null_mut::<AnyObject>()];
                }
            }
        }

        #[method(quitClipFlow:)]
        fn quit_clip_flow(&self, _sender: &AnyObject) {
            if let Some(mtm) = MainThreadMarker::new() {
                unsafe { NSApplication::sharedApplication(mtm).terminate(None) };
            }
        }
    }
);

extern_methods!(
    unsafe impl MenuTarget {
        #[method_id(new)]
        fn new() -> Retained<Self>;
    }
);

static WINDOW: AtomicPtr<AnyObject> = AtomicPtr::new(std::ptr::null_mut());

/// The menu items' target is a weak reference, so the object has to outlive the
/// menu; leaking it once is the simplest way to guarantee that. Declared-class
/// objects are not `Sync`, so the pointer goes in an `AtomicPtr`.
fn target() -> &'static AnyObject {
    static TARGET: AtomicPtr<AnyObject> = AtomicPtr::new(std::ptr::null_mut());
    let ptr = TARGET.load(Ordering::Acquire);
    if !ptr.is_null() {
        return unsafe { &*ptr };
    }
    let obj: Retained<MenuTarget> = MenuTarget::new();
    let ptr = Retained::into_raw(obj) as *mut AnyObject;
    TARGET.store(ptr, Ordering::Release);
    unsafe { &*ptr }
}

/// Remembers the app's window, which the menu needs for showing and hiding.
/// Called from the app's frame loop, once the window exists.
pub fn remember_window(frame: &eframe::Frame) {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    if let Some(w) = frame.winit_window()
        && let Ok(handle) = w.window_handle()
        && let RawWindowHandle::AppKit(appkit) = handle.as_raw()
    {
        // The handle only carries the view; the view knows its window.
        let view = appkit.ns_view.as_ptr() as *mut AnyObject;
        let ns_window: *mut AnyObject = unsafe { msg_send![view, window] };
        if !ns_window.is_null() {
            WINDOW.store(ns_window, Ordering::Release);
        }
    }
}

fn window() -> Option<&'static AnyObject> {
    let ptr = WINDOW.load(Ordering::Acquire);
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { &*ptr })
    }
}

fn icon() -> Option<Retained<NSImage>> {
    // 44x44 template image, drawn once and committed to assets/menu-icon.png.
    const PNG: &[u8] = include_bytes!("../assets/menu-icon.png");
    let data =
        unsafe { NSData::dataWithBytes_length(PNG.as_ptr() as *mut core::ffi::c_void, PNG.len()) };
    let image = unsafe { NSImage::initWithData(NSImage::alloc(), &data)? };
    unsafe {
        image.setSize(NSSize {
            width: 22.0,
            height: 22.0,
        });
        image.setTemplate(true);
    }
    Some(image)
}

/// Adds the status item. Safe to call more than once.
pub fn install() {
    let mtm = match MainThreadMarker::new() {
        Some(m) => m,
        None => return,
    };

    unsafe {
        let bar = NSStatusBar::systemStatusBar();
        // A variable length lets macOS size the item to the icon.
        let item = bar.statusItemWithLength(-1.0);
        item.setVisible(true);

        // `setImage:` is the older spelling of `button.image`; it needs far less
        // of AppKit linked in and does the same thing.
        if let Some(image) = icon() {
            #[allow(deprecated)]
            item.setImage(Some(&image));
        }

        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);

        let toggle = menu.addItemWithTitle_action_keyEquivalent(
            &NSString::from_str("显示 / 隐藏 ClipFlow"),
            Some(sel!(toggleWindow:)),
            &NSString::from_str(""),
        );
        toggle.setTarget(Some(target()));

        let quit = menu.addItemWithTitle_action_keyEquivalent(
            &NSString::from_str("退出 ClipFlow"),
            Some(sel!(quitClipFlow:)),
            &NSString::from_str("q"),
        );
        quit.setTarget(Some(target()));

        item.setMenu(Some(&menu));

        if std::env::var("CLIPFLOW_MENU_DEBUG").is_ok() {
            eprintln!(
                "ClipFlow menu bar: visible={} image={:?} window={}",
                item.isVisible(),
                item.image().map(|i| (i.size().width, i.size().height)),
                window().is_some()
            );
            // Toggle twice, so the self-test covers showing the window again as
            // well as hiding it. The timers run on the main run loop, which is
            // where menu actions belong, and the first waits a moment so the app
            // has had a frame in which to hand over its window.
            let t = target();
            for (delay, sel) in [
                (1.0, sel!(toggleWindow:)),
                (4.0, sel!(toggleWindow:)),
                (7.0, sel!(quitClipFlow:)),
            ] {
                unsafe {
                    NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                        delay, t, sel, None, false,
                    );
                }
            }
        }

        // The status bar keeps the item alive, but leaking it costs nothing and
        // means the menu cannot go away if that ever changes.
        std::mem::forget(item);
    }
}
