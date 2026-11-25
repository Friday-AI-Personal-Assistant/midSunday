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
    pub attention_kv_head_count: Option<u64>, pub attention_key_length: Option<u64>,
    pub attention_value_length: Option<u64>, pub rope_dimension_count: Option<u64>,
    pub expert_count: Option<u64>, pub expert_used_count: Option<u64>,
    pub quantization_version: Option<u32>, pub alignment: Option<u32>,
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
    let(mut attention_kv_head_count,mut attention_key_length,mut attention_value_length,mut rope_dimension_count,mut expert_count,mut expert_used_count,mut quantization_version,mut alignment)=(None,None,None,None,None,None,None,None);
    for _ in 0..entries{
        let key=read_string(&mut f)?;let kind=read_u32(&mut f)?;
        let selected=matches!(key.as_str(),"general.architecture"|"general.name"|"general.file_type"|"general.parameter_count"|"general.quantization_version"|"general.alignment")
            ||key.ends_with(".context_length")||key.ends_with(".block_count")||key.ends_with(".embedding_length")||key.ends_with(".attention.head_count")
            ||key.ends_with(".attention.head_count_kv")||key.ends_with(".attention.key_length")||key.ends_with(".attention.value_length")
            ||key.ends_with(".rope.dimension_count")||key.ends_with(".expert_count")||key.ends_with(".expert_used_count");
        if !selected{skip_value(&mut f,kind)?;continue;}
        let value=scalar(&mut f,kind)?;
        match key.as_str(){
            "general.architecture"=>if let Value::Text(v)=value{architecture=Some(v);},
            "general.name"=>if let Value::Text(v)=value{model_name=Some(v);},
            "general.file_type"=>file_type_code=unsigned(value).and_then(|v|u32::try_from(v).ok()),
            "general.parameter_count"=>parameter_count=unsigned(value),
            "general.quantization_version"=>quantization_version=unsigned(value).and_then(|v|u32::try_from(v).ok()),
            "general.alignment"=>alignment=unsigned(value).and_then(|v|u32::try_from(v).ok()),
            _ if key.ends_with(".context_length")=>context_length=unsigned(value),
            _ if key.ends_with(".block_count")=>block_count=unsigned(value),
            _ if key.ends_with(".embedding_length")=>embedding_length=unsigned(value),
            _ if key.ends_with(".attention.head_count_kv")=>attention_kv_head_count=unsigned(value),
            _ if key.ends_with(".attention.key_length")=>attention_key_length=unsigned(value),
            _ if key.ends_with(".attention.value_length")=>attention_value_length=unsigned(value),
            _ if key.ends_with(".attention.head_count")=>attention_head_count=unsigned(value),
            _ if key.ends_with(".rope.dimension_count")=>rope_dimension_count=unsigned(value),
            _ if key.ends_with(".expert_count")=>expert_count=unsigned(value),
            _ if key.ends_with(".expert_used_count")=>expert_used_count=unsigned(value),
            _=>{}
        }
    }
    Ok(GgufMetadata{format_version:version,file_size_bytes:metadata.len(),tensor_count:tensors,metadata_entry_count:entries,architecture,model_name,file_type_code,file_type_description:file_type_code.and_then(file_type_description),parameter_count,context_length,block_count,embedding_length,attention_head_count,attention_kv_head_count,attention_key_length,attention_value_length,rope_dimension_count,expert_count,expert_used_count,quantization_version,alignment})
}
/// Conservative KV-cache estimate for the configured context, assuming FP16 (2-byte) K/V values.
/// This excludes model weights, allocator overhead, compute buffers, graph/workspace memory, and runtime-specific padding.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KvCacheEstimate {
    pub status: String, pub context_tokens: u64, pub bytes_per_element: u64,
    pub key_cache_bytes: Option<u64>, pub value_cache_bytes: Option<u64>, pub total_bytes: Option<u64>,
    pub assumptions: String,
}

pub fn estimate_kv_cache(metadata: &GgufMetadata, context_tokens: u64) -> Result<KvCacheEstimate, String> {
    if !(512..=32_768).contains(&context_tokens) { return Err("Context length must be between 512 and 32,768 tokens.".to_owned()); }
    const BYTES_PER_ELEMENT: u64 = 2;
    let (Some(layers), Some(kv_heads), Some(key_length), Some(value_length)) = (
        metadata.block_count, metadata.attention_kv_head_count, metadata.attention_key_length, metadata.attention_value_length,
    ) else {
        return Ok(KvCacheEstimate { status: "insufficient_metadata".to_owned(), context_tokens, bytes_per_element: BYTES_PER_ELEMENT,
            key_cache_bytes: None, value_cache_bytes: None, total_bytes: None,
            assumptions: "Requires layer count, KV-head count, and key/value head lengths from GGUF metadata.".to_owned() });
    };
    if layers == 0 || kv_heads == 0 || key_length == 0 || value_length == 0 {
        return Err("GGUF metadata contains a zero-valued KV-cache dimension; no estimate was produced.".to_owned());
    }
    let key_bytes = layers.checked_mul(kv_heads).and_then(|v| v.checked_mul(key_length)).and_then(|v| v.checked_mul(context_tokens)).and_then(|v| v.checked_mul(BYTES_PER_ELEMENT))
        .ok_or_else(|| "KV-cache estimate exceeded the safe numeric range.".to_owned())?;
    let value_bytes = layers.checked_mul(kv_heads).and_then(|v| v.checked_mul(value_length)).and_then(|v| v.checked_mul(context_tokens)).and_then(|v| v.checked_mul(BYTES_PER_ELEMENT))
        .ok_or_else(|| "KV-cache estimate exceeded the safe numeric range.".to_owned())?;
    let total_bytes = key_bytes.checked_add(value_bytes).ok_or_else(|| "Total KV-cache estimate exceeded the safe numeric range.".to_owned())?;
    Ok(KvCacheEstimate { status: "estimated".to_owned(), context_tokens, bytes_per_element: BYTES_PER_ELEMENT,
        key_cache_bytes: Some(key_bytes), value_cache_bytes: Some(value_bytes), total_bytes: Some(total_bytes),
        assumptions: "FP16 K/V cache (2 bytes per element), one sequence, using the configured context length and GGUF-reported dimensions. Actual runtime cache type, batching, parallel sequences, padding, and implementation overhead can change memory use.".to_owned() })
}

#[cfg(test)]
mod tests{
    use super::*;use std::{io::Write,sync::atomic::{AtomicUsize,Ordering}};
    static NEXT:AtomicUsize=AtomicUsize::new(0);
    fn push_string(o:&mut Vec<u8>,s:&str){o.extend_from_slice(&(s.len() as u64).to_le_bytes());o.extend_from_slice(s.as_bytes());}
    fn entry_string(o:&mut Vec<u8>,k:&str,v:&str){push_string(o,k);o.extend_from_slice(&8u32.to_le_bytes());push_string(o,v);}
    fn entry_u32(o:&mut Vec<u8>,k:&str,v:u32){push_string(o,k);o.extend_from_slice(&4u32.to_le_bytes());o.extend_from_slice(&v.to_le_bytes());}
    fn fixture()->Vec<u8>{
        let mut o=b"GGUF".to_vec();o.extend_from_slice(&3u32.to_le_bytes());o.extend_from_slice(&2u64.to_le_bytes());o.extend_from_slice(&16u64.to_le_bytes());
        entry_string(&mut o,"general.architecture","llama");entry_string(&mut o,"general.name","Tiny test model");entry_u32(&mut o,"general.file_type",2);
        push_string(&mut o,"general.parameter_count");o.extend_from_slice(&10u32.to_le_bytes());o.extend_from_slice(&7_000_000_000u64.to_le_bytes());
        entry_u32(&mut o,"llama.context_length",8192);entry_u32(&mut o,"llama.block_count",32);entry_u32(&mut o,"llama.embedding_length",4096);entry_u32(&mut o,"llama.attention.head_count",32);
        entry_u32(&mut o,"llama.attention.head_count_kv",8);
        entry_u32(&mut o,"llama.attention.key_length",128);
        entry_u32(&mut o,"llama.attention.value_length",128);
        entry_u32(&mut o,"llama.rope.dimension_count",128);
        entry_u32(&mut o,"llama.expert_count",8);
        entry_u32(&mut o,"llama.expert_used_count",2);
        entry_u32(&mut o,"general.quantization_version",2);
        entry_u32(&mut o,"general.alignment",32);o
    }
    fn temp(bytes:&[u8],ext:&str)->std::path::PathBuf{let p=std::env::temp_dir().join(format!("midsunday-gguf-{}-{}.{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed),ext));let mut f=File::create(&p).unwrap();f.write_all(bytes).unwrap();p}
    #[test]fn reads_core_metadata_without_tensor_payloads(){let p=temp(&fixture(),"gguf");let m=inspect(p.to_str().unwrap()).unwrap();assert_eq!(m.format_version,3);assert_eq!(m.tensor_count,2);assert_eq!(m.architecture.as_deref(),Some("llama"));assert_eq!(m.model_name.as_deref(),Some("Tiny test model"));assert_eq!(m.file_type_code,Some(2));assert_eq!(m.file_type_description.as_deref(),Some("Q4_0"));assert_eq!(m.parameter_count,Some(7_000_000_000));assert_eq!(m.context_length,Some(8192));assert_eq!(m.block_count,Some(32));assert_eq!(m.embedding_length,Some(4096));assert_eq!(m.attention_head_count,Some(32));assert_eq!(m.attention_kv_head_count,Some(8));assert_eq!(m.attention_key_length,Some(128));assert_eq!(m.attention_value_length,Some(128));assert_eq!(m.rope_dimension_count,Some(128));assert_eq!(m.expert_count,Some(8));assert_eq!(m.expert_used_count,Some(2));assert_eq!(m.quantization_version,Some(2));assert_eq!(m.alignment,Some(32));let _=std::fs::remove_file(p);}
    #[test]fn rejects_invalid_magic_and_versions(){let p=temp(b"NOPE","gguf");assert!(inspect(p.to_str().unwrap()).is_err());let _=std::fs::remove_file(p);let mut b=fixture();b[4..8].copy_from_slice(&99u32.to_le_bytes());let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("version 99"));let _=std::fs::remove_file(p);}
    #[test]fn rejects_truncated_metadata_and_unreasonable_counts(){let mut b=fixture();b.truncate(35);let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).is_err());let _=std::fs::remove_file(p);let mut b=fixture();b[20..28].copy_from_slice(&(MAX_METADATA_ENTRIES+1).to_le_bytes());let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("too many metadata"));let _=std::fs::remove_file(p);}
    #[test]fn skips_unknown_string_and_array_metadata(){let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&2u64.to_le_bytes());push_string(&mut b,"unknown.string");b.extend_from_slice(&8u32.to_le_bytes());push_string(&mut b,"ignored");push_string(&mut b,"unknown.array");b.extend_from_slice(&9u32.to_le_bytes());b.extend_from_slice(&4u32.to_le_bytes());b.extend_from_slice(&3u64.to_le_bytes());for n in [1u32,2,3]{b.extend_from_slice(&n.to_le_bytes());}let p=temp(&b,"gguf");assert_eq!(inspect(p.to_str().unwrap()).unwrap().tensor_count,0);let _=std::fs::remove_file(p);}
    #[test]fn estimates_fp16_kv_cache_from_architecture_dimensions(){
        let p=temp(&fixture(), "gguf"); let metadata=inspect(p.to_str().unwrap()).unwrap();
        let estimate=estimate_kv_cache(&metadata, 8192).unwrap();
        assert_eq!(estimate.status, "estimated");
        assert_eq!(estimate.key_cache_bytes, Some(536_870_912));
        assert_eq!(estimate.value_cache_bytes, Some(536_870_912));
        assert_eq!(estimate.total_bytes, Some(1_073_741_824));
        let _=std::fs::remove_file(p);
    }
    #[test]fn reports_missing_dimensions_without_guessing(){
        let p=temp(&fixture(), "gguf"); let mut metadata=inspect(p.to_str().unwrap()).unwrap(); metadata.attention_value_length=None;
        let estimate=estimate_kv_cache(&metadata, 2048).unwrap();
        assert_eq!(estimate.status, "insufficient_metadata"); assert_eq!(estimate.total_bytes, None);
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_invalid_context_zero_dimensions_and_overflow(){
        let p=temp(&fixture(), "gguf"); let mut metadata=inspect(p.to_str().unwrap()).unwrap();
        assert!(estimate_kv_cache(&metadata, 0).is_err());
        metadata.block_count=Some(0); assert!(estimate_kv_cache(&metadata, 2048).is_err());
        metadata.block_count=Some(u64::MAX); assert!(estimate_kv_cache(&metadata, 32768).is_err());
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_non_gguf_extensions(){let p=temp(&fixture(),"bin");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains(".gguf"));let _=std::fs::remove_file(p);}
}