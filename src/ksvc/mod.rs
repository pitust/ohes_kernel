use crate::prelude::*;

ezy_static! { KSVC_TABLE, BTreeMap<String, Box<dyn Send + Sync + Fn<(), Output = ()>>>, BTreeMap::new() }


pub fn ksvc_init() {
    
}