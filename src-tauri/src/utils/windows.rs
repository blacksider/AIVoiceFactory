use anyhow::{anyhow, Result};
use std::ffi::OsString;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::slice;

use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::processthreadsapi::TerminateProcess;
use winapi::um::psapi::EnumProcesses;
use winapi::um::psapi::GetProcessImageFileNameW;
use winapi::um::winnt::{HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE};

#[cfg(target_os = "windows")]
pub fn process_exists(name: &str, path: &str) -> (bool, DWORD) {
    unsafe {
        let mut processes: [DWORD; 1024] = [0; 1024];
        let mut bytes_returned: DWORD = 0;

        // Enumerate all running processes
        if EnumProcesses(
            processes.as_mut_ptr(),
            size_of::<DWORD>() as DWORD * processes.len() as DWORD,
            &mut bytes_returned,
        ) == 0
        {
            return (false, 0);
        }

        // Calculate the number of processes returned
        let process_count = bytes_returned / size_of::<DWORD>() as DWORD;

        // Loop through the list of process IDs and check each one
        for i in 0..process_count {
            let process_id = processes[i as usize];
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, process_id);

            if !handle.is_null() {
                let mut image_path: Vec<u16> = Vec::new();
                image_path.resize(4096, 0);

                // Get the image path for the process
                let length = GetProcessImageFileNameW(
                    handle,
                    image_path.as_mut_ptr(),
                    image_path.len() as DWORD,
                );

                if length > 0 {
                    let path_string = OsString::from_wide(slice::from_raw_parts(
                        image_path.as_ptr(),
                        length as usize,
                    ));

                    let path_string = path_string.to_str().unwrap();
                    if path_string.contains(path) {
                        let name_string = PathBuf::from(path_string)
                            .file_name()
                            .unwrap()
                            .to_os_string();
                        let name_local = name_string.to_str().unwrap();
                        if name == name_local {
                            return (true, process_id);
                        }
                    }
                }

                // Close the handle to the process
                // Note: This is very important to prevent handle leaks
                //       and system instability.
                CloseHandle(handle);
            }
        }
    }

    (false, 0)
}

#[cfg(target_os = "windows")]
pub fn terminate_process(pid: DWORD) -> Result<()> {
    let handle: HANDLE = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if handle.is_null() {
        return Err(anyhow!("Failed to open process with pid={}", pid));
    }
    let result = unsafe { TerminateProcess(handle, 1) };
    if result == 0 {
        return Err(anyhow!("Failed to terminate process with pid={}", pid));
    }
    unsafe { CloseHandle(handle) };
    Ok(())
}
