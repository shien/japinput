//! COM ClassFactory。DllGetClassObject から呼ばれ、TextService を生成する。

use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::core::*;

use crate::text_service::TextService;

#[implement(IClassFactory)]
pub struct ClassFactory;

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> Result<()> {
        if punkouter.is_some() {
            return Err(Error::from_hresult(CLASS_E_NOAGGREGATION));
        }

        unsafe {
            let service: IUnknown = TextService::new().into();
            service.query(riid, ppvobject).ok()
        }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}
