//! Closed, data-only first-Credit continuation. This parser grants no execution
//! authority: the runtime must validate the complete board, quote and origins.
use serde_json::Value;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum Stage { Menu, Amount }
#[derive(Debug,Clone,PartialEq,Eq)]
pub(crate) struct Descriptor {
    pub source:u16, pub payer:u8, pub stage:Stage, pub amount:u16,
    pub mouse_ids:Vec<u8>, pub minimum:u16, pub maximum:u16,
}
fn fail()->String {"invalid directUnitPlayCreditState".into()}
fn keys(v:&Value, expected:&[&str])->Result<(),String> {
    let o=v.as_object().ok_or_else(fail)?;
    if o.len()!=expected.len() || expected.iter().any(|k|!o.contains_key(*k)) {return Err(fail());}
    Ok(())
}
fn integer(v:&Value,min:u64,max:u64)->Result<u16,String>{
    v.as_u64().filter(|n|*n>=min&&*n<=max).map(|n|n as u16).ok_or_else(fail)
}
pub(crate) fn parse(v:&Value)->Result<Descriptor,String>{
    keys(v,&["version","source","payer","stage","preCreditAmount","capturedMouseIds","menu"])?;
    if v["version"].as_u64()!=Some(1){return Err(fail());}
    let raw=v["source"].as_str().ok_or_else(fail)?;
    let index=raw.strip_prefix('c').and_then(|n|n.parse::<u16>().ok()).filter(|n|*n<255&&format!("c{n}")==raw).ok_or_else(fail)?;
    let payer=match v["payer"].as_str(){Some("p1")=>0,Some("p2")=>1,_=>return Err(fail())};
    let stage=match v["stage"].as_str(){Some("menu")=>Stage::Menu,Some("amount")=>Stage::Amount,_=>return Err(fail())};
    let amount=integer(&v["preCreditAmount"],1,65535)?;
    let ids=v["capturedMouseIds"].as_array().filter(|a|a.len()<=64).ok_or_else(fail)?;
    let mut mouse_ids=Vec::with_capacity(ids.len());let mut previous=0;
    for id in ids {let id=integer(id,1,64)? as u8;if id<=previous{return Err(fail());}mouse_ids.push(id);previous=id;}
    keys(&v["menu"],&["minimum","maximum","canCancel"])?;
    if v["menu"]["canCancel"]!=true{return Err(fail());}
    let minimum=integer(&v["menu"]["minimum"],0,65535)?;
    let maximum=integer(&v["menu"]["maximum"],1,65535)?;
    if minimum>maximum || matches!(stage,Stage::Amount)&&minimum.max(1)>=maximum{return Err(fail());}
    Ok(Descriptor{source:index+1,payer,stage,amount,mouse_ids,minimum,maximum})
}
