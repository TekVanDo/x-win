#![deny(unused_imports)]

use std::{
  fs::{read_link, File},
  io::Read,
};

use xcb::{x, Xid};

use crate::common::{
  api::API,
  x_win_struct::{
    process_info::ProcessInfo, usage_info::UsageInfo, window_info::WindowInfo,
    window_position::WindowPosition,
  },
};

pub struct LinuxAPI {}

/**
 * Impl. for windows system
 */
impl API for LinuxAPI {
  fn get_active_window(&self) -> Result<WindowInfo, napi::Error> {
    let (conn, _) = xcb::Connection::connect(None).unwrap();

    let setup = conn.get_setup();

    let root_window = setup.roots().next();
    if !root_window.is_none() {
      let root_window = root_window.unwrap().root();

      if let Ok(active_window_atom) = get_active_window_atom(&conn) {
        if active_window_atom != x::ATOM_NONE {
          let active_window = conn.send_request(&x::GetProperty {
            delete: false,
            window: root_window,
            property: active_window_atom,
            r#type: x::ATOM_WINDOW,
            long_offset: 0,
            long_length: 1,
          });
          if let Ok(active_window) = conn.wait_for_reply(active_window) {
            let active_window: Option<&x::Window> = active_window.value::<x::Window>().get(0);
            if !active_window.is_none() {
              let active_window: &x::Window = active_window.unwrap();
              return Ok(get_window_information(&conn, active_window));
            }
          }
        }
      }
    }

    Ok(WindowInfo {
      id: 0,
      os: os_name(),
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
    })
  }

  fn get_open_windows(&self) -> Result<Vec<WindowInfo>, napi::Error> {
    let mut results: Vec<WindowInfo> = Vec::new();

    let (conn, _) = xcb::Connection::connect(None).unwrap();

    let setup = conn.get_setup();

    let root_window = setup.roots().next();
    if !root_window.is_none() {
      let root_window = root_window.unwrap().root();

      if let Ok(open_windows_atom) = get_client_list_stacking_atom(&conn) {
        if open_windows_atom != x::ATOM_NONE {
          let window_list = conn.send_request(&x::GetProperty {
            delete: false,
            window: root_window,
            property: open_windows_atom,
            r#type: x::ATOM_WINDOW,
            long_offset: 0,
            long_length: std::u32::MAX,
          });
          if let Ok(windows_reply) = conn.wait_for_reply(window_list) {
            let window_list: Vec<x::Window> = windows_reply.value::<x::Window>().to_vec();
            if window_list.len().ne(&0) {
              for window in window_list {
                let window: &x::Window = &window;
                let result = get_window_information(&conn, window);
                if result.id.ne(&0) {
                  if is_normal_window(&conn, *window) {
                    results.push(result);
                  }
                }
              }
            }
          }
        }
      }
    }
    Ok(results)
  }
}

/**
 * To know the os
 */
fn os_name() -> String {
  r#"linux"#.to_owned()
}

/**
 * Get window information
 */
fn get_window_information(conn: &xcb::Connection, window: &x::Window) -> WindowInfo {
  let window_pid: u32 = get_window_pid(&conn, *window);
  if window_pid != 0 {
    let (path, exec_name) = get_window_path_name(window_pid);
    WindowInfo {
      os: os_name(),
      id: window.resource_id(),
      title: get_window_title(&conn, *window),
      position: get_window_position(&conn, *window),
      info: ProcessInfo {
        process_id: window_pid.try_into().unwrap(),
        path,
        exec_name,
        name: get_window_class_name(&conn, *window),
      },
      usage: UsageInfo {
        memory: get_window_memory_usage(window_pid),
      },
    }
  } else {
    WindowInfo {
      id: 0,
      os: os_name(),
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
    }
  }
}

/**
 * Get active window
 */
fn get_active_window_atom(conn: &xcb::Connection) -> xcb::Result<x::Atom> {
  let active_window_id = conn.send_request(&x::InternAtom {
    only_if_exists: true,
    name: b"_NET_ACTIVE_WINDOW",
  });
  Ok(conn.wait_for_reply(active_window_id)?.atom())
}

/**
 * Get list of open windows
 */
fn get_client_list_stacking_atom(conn: &xcb::Connection) -> xcb::Result<x::Atom> {
  let active_window_id = conn.send_request(&x::InternAtom {
    only_if_exists: true,
    name: b"_NET_CLIENT_LIST_STACKING",
  });
  Ok(conn.wait_for_reply(active_window_id)?.atom())
}

/**
 * Get pid
 */
fn get_window_pid(conn: &xcb::Connection, window: x::Window) -> u32 {
  let window_pid = conn.send_request(&x::InternAtom {
    only_if_exists: true,
    name: b"_NET_WM_PID",
  });
  if let Ok(window_pid) = conn.wait_for_reply(window_pid) {
    let window_pid = conn.send_request(&x::GetProperty {
      delete: false,
      window,
      property: window_pid.atom(),
      r#type: x::ATOM_ANY,
      long_offset: 0,
      long_length: 1,
    });
    if let Ok(window_pid) = conn.wait_for_reply(window_pid) {
      return window_pid.value::<u32>().get(0).unwrap_or(&0).to_owned();
    }
  }
  return 0;
}

/**
 * Get window width, height, x and y
 */
fn get_window_position(conn: &xcb::Connection, window: x::Window) -> WindowPosition {
  let mut position = WindowPosition {
    x: 0,
    y: 0,
    width: 0,
    height: 0,
  };
  let window_geometry = conn.send_request(&x::GetGeometry {
    drawable: x::Drawable::Window(window),
  });
  if let Ok(window_geometry) = conn.wait_for_reply(window_geometry) {
    position.height = window_geometry.height() as i32;
    position.width = window_geometry.width() as i32;
    let window_geometry_x = window_geometry.x();
    let window_geometry_y = window_geometry.y();
    let translated_position = conn.send_request(&x::TranslateCoordinates {
      dst_window: window_geometry.root(),
      src_window: window,
      src_x: window_geometry_x,
      src_y: window_geometry_y,
    });
    if let Ok(translated_position) = conn.wait_for_reply(translated_position) {
      position.x = (translated_position.dst_x() - window_geometry_x) as i32;
      position.y = (translated_position.dst_y() - window_geometry_y) as i32;
    }
  }

  position
}

/**
 * Get window title
 */
fn get_window_title(conn: &xcb::Connection, window: x::Window) -> String {
  let window_title = conn.send_request(&x::GetProperty {
    delete: false,
    window,
    property: x::ATOM_WM_NAME,
    r#type: x::ATOM_ANY,
    long_offset: 0,
    long_length: std::u32::MAX,
  });
  if let Ok(window_title) = conn.wait_for_reply(window_title) {
    let window_title: &[u8] = window_title.value();
    return xcb::Lat1Str::from_bytes(window_title).to_string().to_owned();
  }
  return "".to_owned();
}

/**
 * Get process name
 */
fn get_window_class_name(conn: &xcb::Connection, window: x::Window) -> String {
  let window_class = conn.send_request(&x::GetProperty {
    delete: false,
    window,
    property: x::ATOM_WM_CLASS,
    r#type: x::ATOM_STRING,
    long_offset: 0,
    long_length: std::u32::MAX,
  });
  if let Ok(window_class) = conn.wait_for_reply(window_class) {
    let window_class = window_class.value();
    let window_class = std::str::from_utf8(window_class);

    let mut process_name = window_class
      .unwrap_or("")
      .split('\u{0}')
      .filter(|str| str.len() > 0)
      .collect::<Vec<&str>>();
    return process_name.pop().unwrap_or("").to_owned();
  }
  return "".to_owned();
}

/**
 * Get usage memory of window from proc
 */
fn get_window_memory_usage(pid: u32) -> u32 {
  let mut statm_file = File::open(format!("/proc/{}/statm", pid)).unwrap();
  let mut statm_content = String::new();
  statm_file.read_to_string(&mut statm_content).unwrap();
  let statm_parts: Vec<&str> = statm_content.split(" ").collect();
  return statm_parts[0].parse().unwrap();
}

/**
 * Recover path and name of application from proc
 */
fn get_window_path_name(pid: u32) -> (String, String) {
  let executable_path = read_link(format!("/proc/{}/exe", pid)).unwrap();
  let path = executable_path.display().to_string();
  let name = executable_path.file_name().unwrap();
  let name = name.to_string_lossy().to_string();
  return (path, name);
}

/**
 * Generate Atom of _NET_WM_WINDOW_TYPE value
 */
fn get_window_type_atom(conn: &xcb::Connection) -> xcb::Result<x::Atom> {
  let state_window = conn.send_request(&x::InternAtom {
    only_if_exists: true,
    name: b"_NET_WM_WINDOW_TYPE",
  });
  Ok(conn.wait_for_reply(state_window)?.atom())
}

/**
 * Generate Atom of _NET_WM_WINDOW_TYPE_NORMAL value
 */
fn get_window_type_normal_atom(conn: &xcb::Connection) -> xcb::Result<x::Atom> {
  let invisible = conn.send_request(&x::InternAtom {
    only_if_exists: true,
    name: b"_NET_WM_WINDOW_TYPE_NORMAL",
  });
  Ok(conn.wait_for_reply(invisible)?.atom())
}

/**
 * Check if the window is a normal type
 */
fn is_normal_window(conn: &xcb::Connection, window: x::Window) -> bool {
  if let Ok(state_window_atom) = get_window_type_atom(&conn) {
    if state_window_atom != x::ATOM_NONE {
      if let Ok(type_normal_atom) = get_window_type_normal_atom(&conn) {
        let window_state = conn.send_request(&x::GetProperty {
          delete: false,
          window,
          property: state_window_atom,
          r#type: x::ATOM_ATOM,
          long_offset: 0,
          long_length: std::u32::MAX,
        });
        if let Ok(window_state) = conn.wait_for_reply(window_state) {
          return window_state.value().contains(&type_normal_atom);
        }
      }
    }
  }
  return false;
}
