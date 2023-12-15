use std::ffi::CString;
use std::io::Error;
use std::io::ErrorKind;
use std::ptr;

use winapi::shared::minwindef::FALSE;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::shellapi::{
    ShellExecuteExA, SEE_MASK_FLAG_DDEWAIT, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOCLOSEPROCESS,
    SHELLEXECUTEINFOA,
};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::wincon::{AttachConsole, FreeConsole};
use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};
use winapi::um::winuser::SW_HIDE;

pub(crate) unsafe fn start_admin_instance(
    file_name: String,
    args: Vec<String>,
    working_dir: String,
) -> Result<(), std::io::Error> {
    let file_name = CString::new(file_name).unwrap();
    let working_dir = CString::new(working_dir).unwrap();
    let args = CString::new(args.join(" ")).unwrap();

    let mut info = SHELLEXECUTEINFOA {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOA>() as u32,
        lpFile: file_name.as_ptr() as *const i8,
        hwnd: ptr::null_mut(),
        lpVerb: "runas\0".as_ptr() as *const i8, // This will promt the UAC dialog
        lpParameters: args.as_ptr() as *const i8,
        lpDirectory: working_dir.as_ptr() as *const i8,

        // I got these from the C# code, not sure if they are needed or what they do.
        fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_FLAG_DDEWAIT | SEE_MASK_FLAG_NO_UI,
        dwHotKey: 0,
        hMonitor: ptr::null_mut(),
        hProcess: ptr::null_mut(),
        hInstApp: ptr::null_mut(),
        lpIDList: ptr::null_mut(),
        lpClass: ptr::null_mut(),
        hkeyClass: ptr::null_mut(),
        nShow: SW_HIDE, // This is an important one, it hides the new process window
    };

    // Start the process
    if ShellExecuteExA(&mut info) == FALSE {
        // get last win32 error
        let error = std::io::Error::last_os_error();
        return Err(error);
    }

    if info.hProcess.is_null() {
        return Err(Error::new(ErrorKind::Other, "hProcess was null"));
    }

    // If it has got here then the child process has been started and it *should* have
    // bound itself to this console window. Including stdin, stdout and stderr.
    WaitForSingleObject(info.hProcess, INFINITE);
    let error = std::io::Error::last_os_error();
    if error.raw_os_error().unwrap() != 0 {
        return Err(error);
    }

    println!("Child process has exited");
    Ok(())
}

pub(crate) unsafe fn bind_console(process_id: u32) -> Result<(), Error> {
    if FreeConsole() == FALSE {
        let error = std::io::Error::last_os_error();
        return Err(error);
    }

    if AttachConsole(process_id) == FALSE {
        let error = std::io::Error::last_os_error();
        return Err(error);
    }

    return Ok(());
}

pub(crate) unsafe fn is_admin() -> Result<bool, Error> {
    let mut token: HANDLE = ptr::null_mut();

    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
        let error = std::io::Error::last_os_error();
        return Err(error);
    }

    let mut token_info = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut return_length: u32 = 0;

    if GetTokenInformation(
        token,
        TokenElevation,
        &mut token_info as *mut _ as *mut _,
        std::mem::size_of::<TOKEN_ELEVATION>() as u32,
        &mut return_length,
    ) == FALSE
    {
        let error = std::io::Error::last_os_error();
        return Err(error);
    }

    Ok(token_info.TokenIsElevated > 0)
}
