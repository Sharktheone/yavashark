use base64::alphabet::STANDARD;
use yavashark_macro::{object, props};
use crate::{Error, ObjectHandle, Res};

#[object]
#[derive(Debug)]
pub struct Uint8Array {}

#[props]
impl Uint8Array {
    #[prop("fromBase64")]
    fn from_base_64(base64: &str, options: Option<ObjectHandle>) -> Res<ObjectHandle> {
        Err(Error::new("Not implemented"))
    }
    
    #[prop("fromHex")]
    fn from_hex(hex: &str) -> Res<ObjectHandle> {
        Err(Error::new("Not implemented"))
    }
    
    #[prop("setFromBase64")]
    fn set_from_base_64(&self, base64: &str, options: Option<ObjectHandle>) -> Res<()> {
        Err(Error::new("Not implemented"))
    }
    
    #[prop("toBase64")]
    fn to_base_64(&self, options: Option<ObjectHandle>) -> Res<String> {
        Err(Error::new("Not implemented"))
    }
    
    #[prop("toHex")]
    fn to_hex(&self) -> Res<String> {
        Err(Error::new("Not implemented"))
    }
    
    #[prop("setFromHex")]
    fn set_from_hex(&self, hex: &str) -> Res<()> {
        Err(Error::new("Not implemented"))
    }
}

