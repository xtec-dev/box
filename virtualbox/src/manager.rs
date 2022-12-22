// https://github.com/microsoft/windows-rs/blob/master/docs/FAQ.md
// https://www.reddit.com/r/rust/comments/wl0z41/comment/ijt9071/

// https://learn.microsoft.com/en-us/windows/dev-environment/rust/rss-reader-rust-for-windows



// https://learn.microsoft.com/en-us/windows/win32/com/using-and-implementing-iunknown

use anyhow::Result;
use windows::core::*;
use windows::Win32::System::Com::*;

/*
#[windows::core::interface("Virtualbox.Virtualbox")]
unsafe trait Virtualbox: IUnknown {
    //unsafe fn MyFunction(&self) -> windows::core::HRESULT;
}
*/

//unsafe trait IVirtualBoxClient {}


pub const CUIVirtualBoxClient: GUID = GUID::from_u128(0xdd3fc71d_26c0_4fe1_bf6f_67f633265bba);

trait IVirtualBoxClient{}

//#[windows::core::implement(IVirtualBoxClient)]
struct VirtualBoxClient();
/* 
#[allow(non_snake_case)]
impl IVirtualBoxClient_Impl for VirtualBoxClient {
    // fn CreateView(&self) -> Result<()> {}
}*/
 

fn start() -> Result<()> {
    
    unsafe {
        CoInitialize(None)?;
        //let i:VirtualBoxClient = CoCreateInstance(&CUIVirtualBoxClient, None, CLSCTX_INPROC_SERVER)?;    
        CoUninitialize();
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
