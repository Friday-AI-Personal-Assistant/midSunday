//! Bounded, read-only inspection of GGUF container metadata.
//! This parser stops before tensor descriptors and payloads; model weights are never loaded.
use serde::Serialize;
use std::{fs::File, io::{Read, Seek, SeekFrom}, path::Path};

const MAX_METADATA_ENTRIES: u64 = 100_000;
const MAX_STRING_BYTES: u64 = 1_048_576;
const MAX_ARRAY_ITEMS: u64 = 10_000_000;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GgufMetadata {
    pub format_version: u32, pub file_size_bytes: u64, pub tensor_count: u64,
    pub metadata_entry_count: u64, pub architecture: Option<String>, pub model_name: Option<String>,
    pub file_type_code: Option<u32>, pub file_type_description: Option<String>,
    pub parameter_count: Option<u64>, pub context_length: Option<u64>, pub block_count: Option<u64>,
    pub embedding_length: Option<u64>, pub attention_head_count: Option<u64>,
}
#[derive(Debug)]
enum Value { Text(String), Unsigned(u64), Signed(i64), Other }

fn read_u8(f:&mut File)->Result<u8,String>{let mut b=[0;1];f.read_exact(&mut b).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;Ok(b[0])}
fn read_u32(f:&mut File)->Result<u32,String>{let mut b=[0;4];f.read_exact(&mut b).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;Ok(u32::from_le_bytes(b))}
fn read_u64(f:&mut File)->Result<u64,String>{let mut b=[0;8];f.read_exact(&mut b).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;Ok(u64::from_le_bytes(b))}
fn read_string(f:&mut File)->Result<String,String>{
    let len=read_u64(f)?; if len>MAX_STRING_BYTES{return Err("A GGUF metadata string exceeds the safe inspection limit.".to_owned());}
    let size=usize::try_from(len).map_err(|_|"GGUF metadata string is too large for this device.".to_owned())?;
    let mut bytes=vec![0;size];f.read_exact(&mut bytes).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;
    String::from_utf8(bytes).map_err(|_|"The GGUF metadata contains an invalid UTF-8 string.".to_owned())
}
fn skip_bytes(f:&mut File,count:u64)->Result<(),String>{
    let count=i64::try_from(count).map_err(|_|"GGUF metadata field is too large.".to_owned())?;
    let pos=f.seek(SeekFrom::Current(count)).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;
    if pos>f.metadata().map_err(|_|"Could not inspect GGUF file length.".to_owned())?.len(){return Err("The GGUF metadata is truncated.".to_owned());}
    Ok(())
}
fn scalar(f:&mut File,kind:u32)->Result<Value,String>{
    match kind {
        0=>Ok(Value::Unsigned(read_u8(f)? as u64)),
        1=>Ok(Value::Signed(read_u8(f)? as i8 as i64)),
        2=>{let mut b=[0;2];f.read_exact(&mut b).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;Ok(Value::Unsigned(u16::from_le_bytes(b) as u64))},
        3=>{let mut b=[0;2];f.read_exact(&mut b).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;Ok(Value::Signed(i16::from_le_bytes(b) as i64))},
        4=>Ok(Value::Unsigned(read_u32(f)? as u64)),
        5=>Ok(Value::Signed(read_u32(f)? as i32 as i64)),
        6=>{let _=read_u32(f)?;Ok(Value::Other)},
        7=>{let _=read_u8(f)?;Ok(Value::Other)},
        8=>Ok(Value::Text(read_string(f)?)),
        10=>Ok(Value::Unsigned(read_u64(f)?)),
        11=>Ok(Value::Signed(read_u64(f)? as i64)),
        12=>{let _=read_u64(f)?;Ok(Value::Other)},
        _=>Err(format!("The GGUF file uses unsupported metadata value type {kind}.")),
    }
}
fn skip_value(f:&mut File,kind:u32)->Result<(),String>{
    if kind!=9{let _=scalar(f,kind)?;return Ok(());}
    let element=read_u32(f)?;let count=read_u64(f)?;
    if count>MAX_ARRAY_ITEMS{return Err("A GGUF metadata array exceeds the safe inspection limit.".to_owned());}
    if element==9{return Err("Nested GGUF metadata arrays are not supported.".to_owned());}
    for _ in 0..count{match element{
        0|1|7=>skip_bytes(f,1)?,2|3=>skip_bytes(f,2)?,4|5|6=>skip_bytes(f,4)?,
        10|11|12=>skip_bytes(f,8)?,8=>{let _=read_string(f)?;},
        _=>return Err(format!("The GGUF file uses unsupported array element type {element}.")),
    }}
    Ok(())
}
fn unsigned(v:Value)->Option<u64>{match v{Value::Unsigned(n)=>Some(n),Value::Signed(n) if n>=0=>Some(n as u64),_=>None}}
fn file_type_description(code:u32)->Option<String>{
    let label=match code{0=>"F32",1=>"F16",2=>"Q4_0",3=>"Q4_1",4=>"Q4_1 with some F16 tensors",5=>"Q8_0",6=>"Q5_0",7=>"Q5_1",8=>"Q2_K",9=>"Q3_K_S",10=>"Q3_K_M",11=>"Q3_K_L",12=>"Q4_K_S",13=>"Q4_K_M",14=>"Q5_K_S",15=>"Q5_K_M",16=>"Q6_K",17=>"Q8_K",18=>"IQ2_XXS",19=>"IQ2_XS",20=>"IQ3_XXS",21=>"IQ1_S",22=>"IQ4_NL",23=>"IQ3_S",24=>"IQ2_S",25=>"IQ4_XS",_=>return None};Some(label.to_owned())
}
pub fn inspect(input:&str)->Result<GgufMetadata,String>{
    let path=Path::new(input).canonicalize().map_err(|_|"The selected GGUF file could not be found.".to_owned())?;
    if !path.is_file(){return Err("The selected GGUF path is not a file.".to_owned());}
    if !path.extension().and_then(|v|v.to_str()).is_some_and(|v|v.eq_ignore_ascii_case("gguf")){return Err("Only .gguf files can be inspected.".to_owned());}
    let metadata=std::fs::metadata(&path).map_err(|e|format!("Could not inspect the GGUF file: {e}"))?;
    let mut f=File::open(&path).map_err(|e|format!("Could not open the GGUF file: {e}"))?;
    let mut magic=[0;4];f.read_exact(&mut magic).map_err(|_|"The selected file is too small to be a GGUF file.".to_owned())?;
    if &magic!=b"GGUF"{return Err("The selected file does not have a valid GGUF header.".to_owned());}
    let version=read_u32(&mut f)?;if !(2..=3).contains(&version){return Err(format!("GGUF version {version} is not supported by this metadata inspector (supported: 2 and 3)."));}
    let tensors=read_u64(&mut f)?;let entries=read_u64(&mut f)?;
    if entries>MAX_METADATA_ENTRIES{return Err("The GGUF file declares too many metadata entries for safe inspection.".to_owned());}
    let(mut architecture,mut model_name,mut file_type_code,mut parameter_count,mut context_length,mut block_count,mut embedding_length,mut attention_head_count)=(None,None,None,None,None,None,None,None);
    for _ in 0..entries{
        let key=read_string(&mut f)?;let kind=read_u32(&mut f)?;
        let selected=matches!(key.as_str(),"general.architecture"|"general.name"|"general.file_type"|"general.parameter_count")
            ||key.ends_with(".context_length")||key.ends_with(".block_count")||key.ends_with(".embedding_length")||key.ends_with(".attention.head_count");
        if !selected{skip_value(&mut f,kind)?;continue;}
        let value=scalar(&mut f,kind)?;
        match key.as_str(){
            "general.architecture"=>if let Value::Text(v)=value{architecture=Some(v);},
            "general.name"=>if let Value::Text(v)=value{model_name=Some(v);},
            "general.file_type"=>file_type_code=unsigned(value).and_then(|v|u32::try_from(v).ok()),
            "general.parameter_count"=>parameter_count=unsigned(value),
            _ if key.ends_with(".context_length")=>context_length=unsigned(value),
            _ if key.ends_with(".block_count")=>block_count=unsigned(value),
            _ if key.ends_with(".embedding_length")=>embedding_length=unsigned(value),
            _ if key.ends_with(".attention.head_count")=>attention_head_count=unsigned(value),
            _=>{}
        }
    }
    Ok(GgufMetadata{format_version:version,file_size_bytes:metadata.len(),tensor_count:tensors,metadata_entry_count:entries,architecture,model_name,file_type_code,file_type_description:file_type_code.and_then(file_type_description),parameter_count,context_length,block_count,embedding_length,attention_head_count})
}
#[cfg(test)]
mod tests{
    use super::*;use std::{io::Write,sync::atomic::{AtomicUsize,Ordering}};
    static NEXT:AtomicUsize=AtomicUsize::new(0);
    fn push_string(o:&mut Vec<u8>,s:&str){o.extend_from_slice(&(s.len() as u64).to_le_bytes());o.extend_from_slice(s.as_bytes());}
    fn entry_string(o:&mut Vec<u8>,k:&str,v:&str){push_string(o,k);o.extend_from_slice(&8u32.to_le_bytes());push_string(o,v);}
    fn entry_u32(o:&mut Vec<u8>,k:&str,v:u32){push_string(o,k);o.extend_from_slice(&4u32.to_le_bytes());o.extend_from_slice(&v.to_le_bytes());}
    fn fixture()->Vec<u8>{
        let mut o=b"GGUF".to_vec();o.extend_from_slice(&3u32.to_le_bytes());o.extend_from_slice(&2u64.to_le_bytes());o.extend_from_slice(&8u64.to_le_bytes());
        entry_string(&mut o,"general.architecture","llama");entry_string(&mut o,"general.name","Tiny test model");entry_u32(&mut o,"general.file_type",2);
        push_string(&mut o,"general.parameter_count");o.extend_from_slice(&10u32.to_le_bytes());o.extend_from_slice(&7_000_000_000u64.to_le_bytes());
        entry_u32(&mut o,"llama.context_length",8192);entry_u32(&mut o,"llama.block_count",32);entry_u32(&mut o,"llama.embedding_length",4096);entry_u32(&mut o,"llama.attention.head_count",32);o
    }
    fn temp(bytes:&[u8],ext:&str)->std::path::PathBuf{let p=std::env::temp_dir().join(format!("midsunday-gguf-{}-{}.{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed),ext));let mut f=File::create(&p).unwrap();f.write_all(bytes).unwrap();p}
    #[test]fn reads_core_metadata_without_tensor_payloads(){let p=temp(&fixture(),"gguf");let m=inspect(p.to_str().unwrap()).unwrap();assert_eq!(m.format_version,3);assert_eq!(m.tensor_count,2);assert_eq!(m.architecture.as_deref(),Some("llama"));assert_eq!(m.model_name.as_deref(),Some("Tiny test model"));assert_eq!(m.file_type_code,Some(2));assert_eq!(m.file_type_description.as_deref(),Some("Q4_0"));assert_eq!(m.parameter_count,Some(7_000_000_000));assert_eq!(m.context_length,Some(8192));assert_eq!(m.block_count,Some(32));assert_eq!(m.embedding_length,Some(4096));assert_eq!(m.attention_head_count,Some(32));let _=std::fs::remove_file(p);}
    #[test]fn rejects_invalid_magic_and_versions(){let p=temp(b"NOPE","gguf");assert!(inspect(p.to_str().unwrap()).is_err());let _=std::fs::remove_file(p);let mut b=fixture();b[4..8].copy_from_slice(&99u32.to_le_bytes());let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("version 99"));let _=std::fs::remove_file(p);}
    #[test]fn rejects_truncated_metadata_and_unreasonable_counts(){let mut b=fixture();b.truncate(35);let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).is_err());let _=std::fs::remove_file(p);let mut b=fixture();b[20..28].copy_from_slice(&(MAX_METADATA_ENTRIES+1).to_le_bytes());let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("too many metadata"));let _=std::fs::remove_file(p);}
    #[test]fn skips_unknown_string_and_array_metadata(){let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&2u64.to_le_bytes());push_string(&mut b,"unknown.string");b.extend_from_slice(&8u32.to_le_bytes());push_string(&mut b,"ignored");push_string(&mut b,"unknown.array");b.extend_from_slice(&9u32.to_le_bytes());b.extend_from_slice(&4u32.to_le_bytes());b.extend_from_slice(&3u64.to_le_bytes());for n in [1u32,2,3]{b.extend_from_slice(&n.to_le_bytes());}let p=temp(&b,"gguf");assert_eq!(inspect(p.to_str().unwrap()).unwrap().tensor_count,0);let _=std::fs::remove_file(p);}
    #[test]fn rejects_non_gguf_extensions(){let p=temp(&fixture(),"bin");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains(".gguf"));let _=std::fs::remove_file(p);}
}