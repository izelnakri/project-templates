pub fn silence_stdout<F: FnOnce()>(f: F) {
    use std::fs::File;
    use std::os::unix::io::{AsRawFd};

    let _ = std::io::stdout();

    let devnull = File::open("/dev/null").unwrap();
    let devnull_fd = devnull.as_raw_fd(); // Don't consume the File
    let stdout_fd = unsafe { libc::dup(libc::STDOUT_FILENO) };

    unsafe {
        libc::dup2(devnull_fd, libc::STDOUT_FILENO);
        f();
        libc::dup2(stdout_fd, libc::STDOUT_FILENO);
        libc::close(stdout_fd);
    }
    // devnull is dropped here and its fd closed safely
}
