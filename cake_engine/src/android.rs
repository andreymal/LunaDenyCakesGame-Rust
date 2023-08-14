//! Штуки, специфичные для платформы Android.

use macroquad::miniquad::native::android::{attach_jni_env, ndk_utils};
use std::ffi::c_void;

#[macro_export]
macro_rules! android_call_static_method {
    ($fn:tt, $env:expr, $class_name:expr, $method:expr, $sig:expr $(, $args:expr)*) => {{
        let find_class = (**$env).FindClass.unwrap();
        let get_static_method_id = (**$env).GetStaticMethodID.unwrap();
        let call_static_fn_method = (**$env).$fn.unwrap();

        let class_name = std::ffi::CString::new($class_name).unwrap();
        let class = find_class($env, class_name.as_ptr() as _);

        let method_name = std::ffi::CString::new($method).unwrap();
        let sig = std::ffi::CString::new($sig).unwrap();

        let static_method = get_static_method_id($env, class, method_name.as_ptr() as _, sig.as_ptr() as _);
        call_static_fn_method($env, class, static_method, $($args,)*)
    }};
}

/// Java-эквивалент: `ActivityThread.currentActivityThread().getApplication()`
pub unsafe fn get_android_context() -> *mut c_void {
    let env = attach_jni_env();
    let at = android_call_static_method!(
        CallStaticObjectMethod,
        env,
        "android/app/ActivityThread",
        "currentActivityThread",
        "()Landroid/app/ActivityThread;"
    );

    ndk_utils::call_object_method!(env, at, "getApplication", "()Landroid/app/Application;")
}

/// Возвращает путь к каталогу, в который Android-приложение может сохранять свои файлы.
///
/// Java-эквивалент: `ActivityThread.currentActivityThread().getApplication().getFilesDir().toString()`
pub fn get_files_dir() -> String {
    unsafe {
        let ctx = get_android_context();
        let env = attach_jni_env();
        let files_dir_file =
            ndk_utils::call_object_method!(env, ctx, "getFilesDir", "()Ljava/io/File;");
        let files_dir_string =
            ndk_utils::call_object_method!(env, files_dir_file, "toString", "()Ljava/lang/String;");
        ndk_utils::get_utf_str!(env, files_dir_string).to_string()
    }
}

pub use android_call_static_method as call_static_method;
