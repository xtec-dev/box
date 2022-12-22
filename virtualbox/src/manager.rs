

// https://github.com/retep998/winapi-rs

// https://github.com/microsoft/windows-rs/blob/master/docs/FAQ.md
// https://www.reddit.com/r/rust/comments/wl0z41/comment/ijt9071/

// https://learn.microsoft.com/en-us/windows/dev-environment/rust/rss-reader-rust-for-windows



// https://learn.microsoft.com/en-us/windows/win32/com/using-and-implementing-iunknown

use anyhow::Result;
use windows::core::*;
use windows::Win32::System::Com::*;

#[windows::core::interface("Virtualbox.Virtualbox")]
unsafe trait Virtualbox: IUnknown {
    //unsafe fn MyFunction(&self) -> windows::core::HRESULT;
}

fn start() -> Result<()> {
    
    unsafe {
        let stream = CreateStreamOnHGlobal(0, true)?;
        let values = vec![1u8, 2u8, 3u8, 4u8];

        let mut copied = 0;
        stream.Write(values.as_ptr() as _, values.len() as _, Some(&mut copied)).ok()?;
        assert!(copied == 4);

        let mut position = 0;
        stream.Seek(0, STREAM_SEEK_SET, Some(&mut position))?;
        assert!(position == 0);

        let mut values = vec![0, 0, 0, 0];
        let mut copied = 0;
        stream.Read(values.as_mut_ptr() as _, values.len() as _, Some(&mut copied)).ok()?;
        assert!(copied == 4);
        assert_eq!(values, vec![1u8, 2u8, 3u8, 4u8]);
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_start() {
        start().unwrap();
    }

}
