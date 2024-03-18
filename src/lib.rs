#![deny(unsafe_op_in_unsafe_fn)]
// #![deny(clippy::all)]
//#![allow(unused_imports)]

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate core;

use common::{api::API, thread::ThreadManager, x_win_struct::window_info::WindowInfo};
use napi::{JsFunction, Result, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

mod common;

#[cfg(target_os = "windows")]
mod win32;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
use win32::init_platform_api;

#[cfg(target_os = "linux")]
use linux::init_platform_api;

#[cfg(target_os = "macos")]
use macos::init_platform_api;

#[macro_use]
extern crate napi_derive;

use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use std::{
  thread,
  time::Duration,
};

use crate::common::x_win_struct::{
  process_info::ProcessInfo, usage_info::UsageInfo, window_position::WindowPosition,
};

use once_cell::sync::Lazy;
use std::sync::Mutex;

static THREAD_MANAGER: Lazy<Mutex<ThreadManager>> = Lazy::new(|| Mutex::new(ThreadManager::new()));

pub struct OpenWindowsTask;
pub struct ActiveWindowTask;

#[cfg(not(target_os = "linux"))]
fn _install_extension() -> () {
  ()
}

#[cfg(not(target_os = "linux"))]
fn _uninstall_extension() -> () {
  ()
}

#[cfg(target_os = "linux")]
fn _install_extension() -> () {
  linux::gnome_install_extension()
}

#[cfg(target_os = "linux")]
fn _uninstall_extension() -> () {
  linux::gnome_uninstall_extension()
}

#[napi]
impl Task for OpenWindowsTask {
  type Output = Vec<WindowInfo>;
  type JsValue = Vec<WindowInfo>;

  fn compute(&mut self) -> Result<Self::Output> {
    open_windows()
  }

  fn resolve(&mut self, _: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

#[napi]
impl Task for ActiveWindowTask {
  type Output = WindowInfo;
  type JsValue = WindowInfo;

  fn compute(&mut self) -> Result<Self::Output> {
    active_window()
  }

  fn resolve(&mut self, _: napi::Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

/**
 * Retrieve information the about currently active window.
 * Returns an object of `WindowInfo`.
 *
 * # Example
 * 
 * ## Javascript example
 * 
 * ```javascript
 * const { activeWindow } = require('@miniben90/x-win');
 * 
 * const currentWindow = activeWindow();
 * console.log(currentWindow);
 * ```
 * 
 * ## Typescript example
 * 
 * ```typescript
 * import { activeWindow } from '@miniben90/x-win';
 * 
 * const currentWindow = activeWindow();
 * console.log(currentWindow);
 * ```
 *
 * # Information about Electron
 *
 * It is recommended to use this function within a worker to mitigate potential recovery issues on MacOS.
 */
#[napi]
pub fn active_window() -> Result<WindowInfo> {
  let api = init_platform_api();
  Ok(api.get_active_window())
}

/**
 * Retrieve information about the currently active window as a promise.
 * Returns an object of `WindowInfo`.
 *
 * # Example
 * 
 * ## Javascript example
 * 
 * ```javascript
 * activeWindowAsync()
 * .then(currentWindow => {
 *   console.log(currentWindow);
 * });
 * ```
 * 
 * ## Typescript example
 *
 * ```typescript
 * import { activeWindowAsync } from '@miniben90/x-win';
 * 
 * activeWindowAsync()
 * .then(currentWindow => {
 *   console.log(currentWindow);
 * });
 * ```
 *
 * # Information about Electron
 *
 * It is recommended to use this function within a worker to mitigate potential recovery issues on MacOS.
 */
#[napi]
pub fn active_window_async() -> AsyncTask<ActiveWindowTask> {
  AsyncTask::new(ActiveWindowTask { })
}

/**
 * Retrieve information about the currently open windows.
 * Returns an array of `WindowInfo`, each containing details about a specific open window.
 *
 * # Example
 * 
 * ## Javascript example
 * 
 * ```javascript
 * const { openWindows } = require('@miniben90/x-win');
 *
 * const windows = openWindows();
 * for (let i = 0; i < windows.length; i++) {
 *   console.log(i, windows[i]);
 * }
 * ```
 * 
 * ## Typescript Example
 *
 * ```typescript
 * import { openWindows } from '@miniben90/x-win';
 *
 * const windows = openWindows();
 * for (let i = 0; i < windows.length; i++) {
 *   console.log(i, windows[i]);
 * }
 * ```
 *
 * # Information about Electron
 *
 * It is recommended to use this function within a worker to mitigate potential recovery issues on MacOS.
 */
#[napi]
pub fn open_windows() -> Result<Vec<WindowInfo>> {
  let api = init_platform_api();
  Ok(api.get_open_windows())
}

/**
 * Retrieve information about the currently open windows as a promise.
 * Returns an array of `WindowInfo`, each containing details about a specific open window.
 *
 * # Example
 * 
 * ## Javascript example
 * 
 * ```javascript
 * const { openWindowsAsync } = resuire('@miniben90/x-win');
 *
 * openWindowsAsync()
 * .then(windows => {
 *   for (let i = 0; i < windows.length; i++) {
 *     console.log(i, windows[i]);
 *   }
 * });
 * ```
 * 
 * ## Typescript example
 * 
 * ```typescript
 * import { openWindowsAsync } from '@miniben90/x-win';
 *
 * openWindowsAsync()
 * .then(windows => {
 *   for (let i = 0; i < windows.length; i++) {
 *     console.log(i, windows[i]);
 *   }
 * });
 * ```
 *
 * # Information about Electron
 *
 * It is recommended to use this function within a worker to mitigate potential recovery issues on MacOS.
 */
#[napi]
pub fn open_windows_async() -> AsyncTask<OpenWindowsTask> {
  AsyncTask::new(OpenWindowsTask { })
}

/**
 * Subscribe an observer thread to monitor changes in the active window.
 *
 * # Example
 * 
 * ## Javascript example
 * 
 * ```javascript
 * const { subscribeActiveWindow, unsubscribeAllActiveWindow } = require('@miniben90/x-win');
 * 
 * const a = subscribeActiveWindow((info) => {
 *   t.log(a, info);
 * });
 * const b = subscribeActiveWindow((info) => {
 *   t.log(b, info);
 * });
 * const c = subscribeActiveWindow((info) => {
 *   t.log(c, info);
 * });
 * 
 * unsubscribeAllActiveWindow();
 * ```
 * 
 * ## Typescript example
 * 
 * ```typescript
 * import { subscribeActiveWindow, unsubscribeAllActiveWindow } from '@miniben90/x-win';
 * 
 * const a = subscribeActiveWindow((info) => {
 *   t.log(a, info);
 * });
 * const b = subscribeActiveWindow((info) => {
 *   t.log(b, info);
 * });
 * const c = subscribeActiveWindow((info) => {
 *   t.log(c, info);
 * });
 * 
 * unsubscribeAllActiveWindow();
 * ```
 *
 */
#[napi(ts_args_type = "callback: (info: WindowInfo) => void")]
pub fn subscribe_active_window(callback: JsFunction) -> Result<u32> {
  let api = init_platform_api();
  let tsfn: ThreadsafeFunction<WindowInfo, ErrorStrategy::Fatal> = callback
    .create_threadsafe_function(
      0,
      |ctx: napi::threadsafe_function::ThreadSafeCallContext<WindowInfo>| Ok(vec![ctx.value]),
    )?;

  let tsfn_clone: ThreadsafeFunction<WindowInfo, ErrorStrategy::Fatal> = tsfn.clone();

  let thread_manager = THREAD_MANAGER.lock().unwrap();

  let id = thread_manager.start_thread(move |receiver| {
    let mut current_window: WindowInfo = WindowInfo {
      id: 0,
      os: "".to_string(),
      title: "".to_string(),
      position: WindowPosition {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
      },
      info: ProcessInfo {
        process_id: 0,
        path: "".to_string(),
        name: "".to_string(),
        exec_name: "".to_string(),
      },
      usage: UsageInfo { memory: 0 },
      url: "".to_string(),
    };
    loop {
      match receiver.try_recv() {
        Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
          break;
        }
        _ => {
          let new_current_window = api.get_active_window();
          if new_current_window.id.ne(&current_window.id)
            || new_current_window.title.ne(&current_window.title)
            || new_current_window
              .info
              .process_id
              .ne(&current_window.info.process_id)
            || new_current_window.id.eq(&0)
          {
            current_window = new_current_window.clone();
            tsfn_clone.call(new_current_window, ThreadsafeFunctionCallMode::Blocking);
          }
          thread::sleep(Duration::from_millis(100));
        }
      }
    }
  });

  Ok(id.unwrap())
}

/**
 * Terminate and unsubscribe a specific observer using their ID.
 *
 * # Example
 * 
 * ## Javascript example
 * 
 * ```javascript
 * const { subscribeActiveWindow, unsubscribeActiveWindow } = require('@miniben90/x-win');
 * 
 * const a = subscribeActiveWindow((info) => {
 *   t.log(a, info);
 * });
 * const b = subscribeActiveWindow((info) => {
 *   t.log(b, info);
 * });
 * const c = subscribeActiveWindow((info) => {
 *   t.log(c, info);
 * });
 * 
 * unsubscribeActiveWindow(a);
 * unsubscribeActiveWindow(b);
 * unsubscribeActiveWindow(c);
 * ```
 * 
 * ## Typescript example
 * 
 * ```typescript
 * import { subscribeActiveWindow, unsubscribeActiveWindow } from '@miniben90/x-win';
 * 
 * const a = subscribeActiveWindow((info) => {
 *   t.log(a, info);
 * });
 * const b = subscribeActiveWindow((info) => {
 *   t.log(b, info);
 * });
 * const c = subscribeActiveWindow((info) => {
 *   t.log(c, info);
 * });
 * 
 * unsubscribeActiveWindow(a);
 * unsubscribeActiveWindow(b);
 * unsubscribeActiveWindow(c);
 * ```
 */
#[napi]
pub fn unsubscribe_active_window(thread_id: u32) -> Result<()> {
  THREAD_MANAGER.lock().unwrap().stop_thread(thread_id).unwrap();
  Ok(())
}

/**
 * Terminate and unsubscribe all observer threads monitoring changes in the active window.
 *
 * # Example
 * 
 * ## Javascript example
 * 
 * ```javascript
 * const { subscribeActiveWindow, unsubscribeAllActiveWindow } = require('@miniben90/x-win');
 * 
 * const a = subscribeActiveWindow((info) => {
 *   t.log(a, info);
 * });
 * const b = subscribeActiveWindow((info) => {
 *   t.log(b, info);
 * });
 * const c = subscribeActiveWindow((info) => {
 *   t.log(c, info);
 * });
 * 
 * unsubscribeAllActiveWindow();
 * ```
 * 
 * ## Typescript example
 * 
 * ```typescript
 * import { subscribeActiveWindow, unsubscribeAllActiveWindow } from '@miniben90/x-win';
 * 
 * const a = subscribeActiveWindow((info) => {
 *   t.log(a, info);
 * });
 * const b = subscribeActiveWindow((info) => {
 *   t.log(b, info);
 * });
 * const c = subscribeActiveWindow((info) => {
 *   t.log(c, info);
 * });
 * 
 * unsubscribeAllActiveWindow();
 * ```
 */
#[napi]
pub fn unsubscribe_all_active_window() -> Result<()> {
  THREAD_MANAGER.lock().unwrap().stop_all_threads().unwrap();
  Ok(())
}

/**
 * Install Gnome extensions required for Linux using Gnome > 41.
 * This function will write extension files needed to correctly detect working windows with Wayland desktop environment.
 *
 * # Example
 * ```javascript
 * const currentWindow = activeWindow();
 * console.log(currentWindow);
 * ```
 *
 * # Information about Electron
 *
 * It is recommended to use this function within a worker to mitigate potential recovery issues on MacOS.
 */
#[napi]
pub fn install_extension() -> () {
  _install_extension()
}

#[napi]
pub fn uninstall_extension() -> () {
  _uninstall_extension()
}