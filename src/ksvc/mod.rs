use crate::prelude::*;
use serde_derive::*;

ezy_static! { KSVC_TABLE, BTreeMap<String, Box<dyn Send + Sync + Fn<(), Output = ()>>>, BTreeMap::new() }

#[derive(Serialize, Deserialize)]
pub enum KSvcResult {
    Success,
    Failure(String),
}

pub fn ksvc_init() {
    let t = KSVC_TABLE.get();
    
    t.insert("log".to_string(), box || {
        let d: String = postcard::from_bytes(preempt::CURRENT_TASK.box1.unwrap()).unwrap();
        print!("{}", d);
        let x = postcard::to_allocvec(&KSvcResult::Success).unwrap();
        preempt::CURRENT_TASK.get().box1 = Some(x.leak());
    });
}