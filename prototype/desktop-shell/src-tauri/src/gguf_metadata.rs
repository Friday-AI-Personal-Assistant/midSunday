//! Bounded, read-only inspection of GGUF container metadata.
//! This parser stops before tensor descriptors and payloads; model weights are never loaded.
use serde::Serialize;
use std::{collections::HashSet, fs::File, io::{Read, Seek, SeekFrom}, path::Path};

const MAX_METADATA_ENTRIES: u64 = 100_000;
const MAX_METADATA_KEY_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TENSORS: u64 = 1_000_000;
// Conservative budget for retained descriptor records and duplicate-name index bookkeeping.
// Tensor-name text has its own separate aggregate byte limit.
const MAX_TENSOR_DESCRIPTOR_MEMORY_BYTES: u64 = 64 * 1024 * 1024;
const ESTIMATED_TENSOR_DESCRIPTOR_BYTES: u64 = 192;
const MAX_TENSOR_NAME_BYTES: u64 = 8 * 1024 * 1024;
const MAX_METADATA_SECTION_BYTES: u64 = 64 * 1024 * 1024;
const MIN_TENSOR_DESCRIPTOR_BYTES: u64 = 33;
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
fn skip_bool(f:&mut File)->Result<(),String>{
    let value=read_u8(f)?;
    if value>1{return Err("GGUF boolean metadata must be encoded as 0 or 1.".to_owned());}
    Ok(())
}
fn read_u32(f:&mut File)->Result<u32,String>{let mut b=[0;4];f.read_exact(&mut b).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;Ok(u32::from_le_bytes(b))}
fn read_u64(f:&mut File)->Result<u64,String>{let mut b=[0;8];f.read_exact(&mut b).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;Ok(u64::from_le_bytes(b))}
fn read_string(f:&mut File)->Result<String,String>{
    read_string_with_limits(f,None,MAX_STRING_BYTES,"A GGUF metadata string exceeds the safe inspection limit.")
}
fn read_string_with_metadata_section_limit(f:&mut File,start:u64)->Result<String,String>{
    let section_end=start.checked_add(MAX_METADATA_SECTION_BYTES)
        .ok_or_else(||"GGUF metadata section limit position overflowed.".to_owned())?;
    read_string_with_limits(f,Some(section_end),MAX_STRING_BYTES,"A GGUF metadata string exceeds the safe inspection limit.")
}
fn read_metadata_key(f:&mut File,remaining_key_budget:u64)->Result<String,String>{
    read_string_with_limits(f,None,remaining_key_budget,"GGUF metadata keys exceed the aggregate safe inspection limit.")
}
fn read_string_with_limits(f:&mut File,section_end:Option<u64>,max_bytes:u64,limit_error:&str)->Result<String,String>{
    let len=read_u64(f)?;
    if len>MAX_STRING_BYTES{return Err("A GGUF metadata string exceeds the safe inspection limit.".to_owned());}
    if len>max_bytes{return Err(limit_error.to_owned());}
    // Check both the opened file boundary and, when scanning a string array, the
    // aggregate metadata-section boundary before allocating or reading the payload.
    let position=f.stream_position().map_err(|_|"Could not locate GGUF string payload.".to_owned())?;
    if let Some(end)=section_end{
        let remaining=end.checked_sub(position).ok_or_else(||"GGUF metadata section exceeds the aggregate safe inspection limit.".to_owned())?;
        if len>remaining{return Err("GGUF metadata section exceeds the aggregate safe inspection limit.".to_owned());}
    }
    let file_len=f.metadata().map_err(|_|"Could not inspect GGUF file length.".to_owned())?.len();
    let remaining=file_len.checked_sub(position).ok_or_else(||"The GGUF metadata is truncated.".to_owned())?;
    if len>remaining{return Err("The GGUF metadata is truncated.".to_owned());}
    let size=usize::try_from(len).map_err(|_|"GGUF metadata string is too large for this device.".to_owned())?;
    let mut bytes=vec![0;size];
    f.read_exact(&mut bytes).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;
    String::from_utf8(bytes).map_err(|_|"The GGUF metadata contains an invalid UTF-8 string.".to_owned())
}
fn skip_bytes(f:&mut File,count:u64)->Result<(),String>{
    let count=i64::try_from(count).map_err(|_|"GGUF metadata field is too large.".to_owned())?;
    let pos=f.seek(SeekFrom::Current(count)).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;
    if pos>f.metadata().map_err(|_|"Could not inspect GGUF file length.".to_owned())?.len(){return Err("The GGUF metadata is truncated.".to_owned());}
    Ok(())
}
fn enforce_metadata_section_limit(f:&mut File,start:u64)->Result<(),String>{
    let current=f.stream_position().map_err(|_|"Could not measure GGUF metadata section length.".to_owned())?;
    let consumed=current.checked_sub(start).ok_or_else(||"GGUF metadata position moved before its section start.".to_owned())?;
    if consumed>MAX_METADATA_SECTION_BYTES{
        return Err("GGUF metadata section exceeds the aggregate safe inspection limit.".to_owned());
    }
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
        7=>{skip_bool(f)?;Ok(Value::Other)},
        8=>Ok(Value::Text(read_string(f)?)),
        10=>Ok(Value::Unsigned(read_u64(f)?)),
        11=>Ok(Value::Signed(read_u64(f)? as i64)),
        12=>{let _=read_u64(f)?;Ok(Value::Other)},
        _=>Err(format!("The GGUF file uses unsupported metadata value type {kind}.")),
    }
}
fn skip_boolean_array(f:&mut File,count:u64)->Result<(),String>{
    let mut remaining=count;
    let mut buffer=[0u8;8192];
    while remaining>0{
        let take=remaining.min(buffer.len() as u64) as usize;
        f.read_exact(&mut buffer[..take]).map_err(|_|"The GGUF metadata is truncated.".to_owned())?;
        if buffer[..take].iter().any(|value|*value>1){
            return Err("GGUF boolean metadata must be encoded as 0 or 1.".to_owned());
        }
        remaining-=take as u64;
    }
    Ok(())
}
fn skip_value(f:&mut File,kind:u32,metadata_section_start:u64)->Result<(),String>{
    if kind!=9{let _=scalar(f,kind)?;return Ok(());}
    let element=read_u32(f)?;let count=read_u64(f)?;
    if count>MAX_ARRAY_ITEMS{return Err("A GGUF metadata array exceeds the safe inspection limit.".to_owned());}
    if element==9{return Err("Nested GGUF metadata arrays are not supported.".to_owned());}
    let width=match element{0|1=>Some(1u64),2|3=>Some(2),4|5|6=>Some(4),10|11|12=>Some(8),_=>None};
    if let Some(width)=width{
        let bytes=count.checked_mul(width).ok_or_else(||"GGUF metadata array byte length overflowed.".to_owned())?;
        return skip_bytes(f,bytes);
    }
    match element{
        7=>skip_boolean_array(f,count),
        // A single string array can otherwise exceed the aggregate metadata-section
        // budget before the caller reaches its normal end-of-entry check.
        8=>{
            for _ in 0..count{
                let _=read_string_with_metadata_section_limit(f,metadata_section_start)?;
                enforce_metadata_section_limit(f,metadata_section_start)?;
            }
            Ok(())
        },
        _=>Err(format!("The GGUF file uses unsupported array element type {element}.")),
    }
}
fn unsigned(v:Value)->Option<u64>{match v{Value::Unsigned(n)=>Some(n),Value::Signed(n) if n>=0=>Some(n as u64),_=>None}}
fn required_unsigned(value:Value,key:&str)->Result<u64,String>{
    unsigned(value).ok_or_else(||format!("GGUF metadata field {key} must contain a non-negative integer."))
}
fn required_u32(value:Value,key:&str)->Result<u32,String>{
    u32::try_from(required_unsigned(value,key)?).map_err(|_|format!("GGUF metadata field {key} exceeds the supported 32-bit range."))
}
fn required_text(value:Value,key:&str)->Result<String,String>{
    match value{Value::Text(text)=>Ok(text),_=>Err(format!("GGUF metadata field {key} must contain a string."))}
}
fn selected_metadata_field(key:&str)->Option<&'static str>{
    match key{
        "general.architecture"=>Some("architecture"),
        "general.name"=>Some("name"),
        "general.file_type"=>Some("file_type"),
        "general.parameter_count"=>Some("parameter_count"),
        "general.quantization_version"=>Some("quantization_version"),
        "general.alignment"=>Some("alignment"),
        _ if key.strip_suffix(".context_length").is_some_and(|prefix|!prefix.is_empty())=>Some("context_length"),
        _ if key.strip_suffix(".block_count").is_some_and(|prefix|!prefix.is_empty())=>Some("block_count"),
        _ if key.strip_suffix(".embedding_length").is_some_and(|prefix|!prefix.is_empty())=>Some("embedding_length"),
        _ if key.strip_suffix(".attention.head_count").is_some_and(|prefix|!prefix.is_empty())=>Some("attention_head_count"),
        _ if key.strip_suffix(".attention.head_count_kv").is_some_and(|prefix|!prefix.is_empty())=>Some("attention_kv_head_count"),
        _ if key.strip_suffix(".attention.key_length").is_some_and(|prefix|!prefix.is_empty())=>Some("attention_key_length"),
        _ if key.strip_suffix(".attention.value_length").is_some_and(|prefix|!prefix.is_empty())=>Some("attention_value_length"),
        _ if key.strip_suffix(".rope.dimension_count").is_some_and(|prefix|!prefix.is_empty())=>Some("rope_dimension_count"),
        _ if key.strip_suffix(".expert_count").is_some_and(|prefix|!prefix.is_empty())=>Some("expert_count"),
        _ if key.strip_suffix(".expert_used_count").is_some_and(|prefix|!prefix.is_empty())=>Some("expert_used_count"),
        _=>None,
    }
}
fn architecture_metadata_prefix(key:&str)->Option<&str>{
    [".context_length",".block_count",".embedding_length",".attention.head_count",".attention.head_count_kv",
     ".attention.key_length",".attention.value_length",".rope.dimension_count",".expert_count",".expert_used_count"]
        .iter().find_map(|suffix|key.strip_suffix(suffix).filter(|prefix|!prefix.is_empty()))
}
fn file_type_description(code:u32)->Option<String>{
    let label=match code{0=>"F32",1=>"F16",2=>"Q4_0",3=>"Q4_1",4=>"Q4_1 with some F16 tensors",5=>"Q8_0",6=>"Q5_0",7=>"Q5_1",8=>"Q2_K",9=>"Q3_K_S",10=>"Q3_K_M",11=>"Q3_K_L",12=>"Q4_K_S",13=>"Q4_K_M",14=>"Q5_K_S",15=>"Q5_K_M",16=>"Q6_K",17=>"Q8_K",18=>"IQ2_XXS",19=>"IQ2_XS",20=>"IQ3_XXS",21=>"IQ1_S",22=>"IQ4_NL",23=>"IQ3_S",24=>"IQ2_S",25=>"IQ4_XS",_=>return None};Some(label.to_owned())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TensorTypeSummary {
    pub type_code: u32,
    pub type_name: String,
    pub tensor_count: u64,
    pub known_payload_bytes: u64,
    pub unknown_size_tensor_count: u64,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TensorStorageAnalysis {
    pub status: String,
    pub tensor_count: u64,
    pub analysed_tensor_count: u64,
    pub unknown_size_tensor_count: u64,
    pub known_payload_bytes: u64,
    pub tensor_element_count: u64,
    pub reported_parameter_count: Option<u64>,
    pub parameter_count_status: String,
    pub data_start_offset: u64,
    pub alignment: u32,
    pub types: Vec<TensorTypeSummary>,
    pub note: String,
}
#[derive(Clone, Debug)]
struct TensorDescriptor { dimensions: [u64;4], dimensions_count: u8, type_code: u32, offset: u64 }

fn tensor_type_layout(code:u32)->Option<(&'static str,u64,u64)> {
    match code {
        0=>Some(("F32",1,4)), 1=>Some(("F16",1,2)),
        2=>Some(("Q4_0",32,18)), 3=>Some(("Q4_1",32,20)),
        6=>Some(("Q5_0",32,22)), 7=>Some(("Q5_1",32,24)),
        8=>Some(("Q8_0",32,34)), 9=>Some(("Q8_1",32,36)),
        10=>Some(("Q2_K",256,84)), 11=>Some(("Q3_K",256,110)),
        12=>Some(("Q4_K",256,144)), 13=>Some(("Q5_K",256,176)),
        14=>Some(("Q6_K",256,210)), 15=>Some(("Q8_K",256,292)),
        32=>Some(("I8",1,1)), 33=>Some(("I16",1,2)),
        34=>Some(("I32",1,4)), 35=>Some(("I64",1,8)),
        36=>Some(("F64",1,8)), 38=>Some(("BF16",1,2)),
        _=>None,
    }
}
fn align_up(value:u64, alignment:u32)->Result<u64,String>{
    let a=u64::from(alignment);
    value.checked_add(a-1).map(|v|v/a*a).ok_or_else(||"GGUF tensor-data alignment overflowed.".to_owned())
}
fn tensor_element_count(dimensions:&[u64])->Result<u64,String>{
    if dimensions.is_empty() || dimensions.iter().any(|value|*value==0){
        return Err("A GGUF tensor has an empty or zero-sized dimension.".to_owned());
    }
    dimensions.iter().try_fold(1u64,|total,value|total.checked_mul(*value)
        .ok_or_else(||"GGUF tensor element count overflowed.".to_owned()))
}
fn tensor_payload_bytes(dimensions:&[u64], block:u64, block_bytes:u64)->Option<u64>{
    let first=*dimensions.first()?;
    if block==0 || block_bytes==0 || first==0 || first%block!=0 || dimensions.iter().any(|v|*v==0){return None;}
    let row=(first/block).checked_mul(block_bytes)?;
    dimensions.iter().skip(1).try_fold(row,|total,v|total.checked_mul(*v))
}
pub fn analyse_tensor_storage(input:&str)->Result<TensorStorageAnalysis,String>{
    let path=Path::new(input).canonicalize().map_err(|_|"The selected GGUF file could not be found.".to_owned())?;
    if !path.is_file() || !path.extension().and_then(|v|v.to_str()).is_some_and(|v|v.eq_ignore_ascii_case("gguf")){
        return Err("Tensor analysis requires an existing .gguf file.".to_owned());
    }
    let mut f=File::open(&path).map_err(|e|format!("Could not open GGUF file: {e}"))?;
    // Read the size from the same opened handle used for parsing, avoiding a
    // path-metadata/open race when checking descriptor offsets against EOF.
    let file_size=f.metadata().map_err(|e|format!("Could not inspect GGUF file: {e}"))?.len();
    let mut magic=[0;4];f.read_exact(&mut magic).map_err(|_|"The GGUF header is truncated.".to_owned())?;
    if &magic!=b"GGUF"{return Err("The selected file does not have a valid GGUF header.".to_owned());}
    let version=read_u32(&mut f)?;
    if !(2..=3).contains(&version){return Err(format!("GGUF version {version} is not supported."));}
    let tensor_count=read_u64(&mut f)?;let metadata_count=read_u64(&mut f)?;
    let metadata_section_start=f.stream_position().map_err(|_|"Could not locate GGUF metadata section.".to_owned())?;
    if metadata_count>MAX_METADATA_ENTRIES{return Err("The GGUF file declares too many metadata entries.".to_owned());}
    if tensor_count>MAX_TENSORS{return Err("The GGUF file declares too many tensors for safe inspection.".to_owned());}
    // Bound the descriptor vector and duplicate-name index before retaining either.
    // The estimate deliberately includes conservative per-entry bookkeeping overhead;
    // tensor-name text is independently bounded by MAX_TENSOR_NAME_BYTES below.
    let estimated_descriptor_bytes=tensor_count.checked_mul(ESTIMATED_TENSOR_DESCRIPTOR_BYTES)
        .ok_or_else(||"GGUF tensor descriptor memory estimate overflowed.".to_owned())?;
    if estimated_descriptor_bytes>MAX_TENSOR_DESCRIPTOR_MEMORY_BYTES{
        return Err("GGUF tensor descriptors exceed the aggregate safe memory budget.".to_owned());
    }
    let mut alignment=32u32;
    let mut reported_parameter_count=None;
    // Bound the aggregate bytes retained by the metadata-key uniqueness set.
    // The per-string cap alone would still allow many large keys to accumulate.
    let mut metadata_keys=HashSet::new();
    let mut metadata_key_bytes=0u64;
    for _ in 0..metadata_count{
        let remaining_key_budget=MAX_METADATA_KEY_BYTES.checked_sub(metadata_key_bytes)
            .ok_or_else(||"GGUF metadata key byte count exceeded its budget.".to_owned())?;
        let key=read_metadata_key(&mut f,remaining_key_budget)?;
        if key.is_empty(){return Err("The GGUF file contains an empty metadata key.".to_owned());}
        metadata_key_bytes=metadata_key_bytes.checked_add(key.len() as u64)
            .ok_or_else(||"GGUF metadata key byte count overflowed.".to_owned())?;
        if metadata_key_bytes>MAX_METADATA_KEY_BYTES{return Err("GGUF metadata keys exceed the aggregate safe inspection limit.".to_owned());}
        if !metadata_keys.insert(key.clone()){return Err("The GGUF file contains a duplicate metadata key.".to_owned());}
        let kind=read_u32(&mut f)?;
        if key=="general.alignment"{
            let value=required_u32(scalar(&mut f,kind)?,&key)?;
            alignment=value;
        }else if key=="general.parameter_count"{
            reported_parameter_count=Some(required_unsigned(scalar(&mut f,kind)?,&key)?);
        }else{skip_value(&mut f,kind,metadata_section_start)?;}
        enforce_metadata_section_limit(&mut f,metadata_section_start)?;
    }
    if alignment==0 || alignment>4096 || !alignment.is_power_of_two(){return Err("GGUF tensor alignment must be a power of two from 1 to 4096 bytes.".to_owned());}
    // A descriptor needs at least: u64 name length, one UTF-8 name byte,
    // u32 dimension count, one u64 dimension, u32 type, and u64 data offset.
    // Check this lower bound against the actual remaining file before allocating
    // descriptor vectors or the duplicate-name set from an untrusted header.
    let descriptor_start=f.stream_position().map_err(|e|format!("Could not locate GGUF tensor descriptors: {e}"))?;
    let minimum_descriptor_bytes=tensor_count.checked_mul(MIN_TENSOR_DESCRIPTOR_BYTES)
        .ok_or_else(||"GGUF tensor descriptor byte count overflowed.".to_owned())?;
    let remaining_file_bytes=file_size.checked_sub(descriptor_start)
        .ok_or_else(||"GGUF tensor descriptor position exceeds the file length.".to_owned())?;
    if minimum_descriptor_bytes>remaining_file_bytes{
        return Err("The declared GGUF tensor descriptors cannot fit within the remaining file bytes.".to_owned());
    }
    // Grow descriptor and name collections incrementally; never reserve directly
    // from an untrusted tensor count. Fixed-size dimensions avoid one heap allocation
    // per tensor, and aggregate name accounting bounds retained name text.
    let mut descriptors=Vec::new();
    let mut tensor_names=HashSet::new();
    let mut tensor_name_bytes=0u64;
    for _ in 0..tensor_count{
        let name_len=read_u64(&mut f)?;
        if name_len==0 || name_len>64{return Err("A GGUF tensor name must contain between 1 and 64 bytes.".to_owned());}
        let name_size=usize::try_from(name_len).map_err(|_|"GGUF tensor name is too large for this device.".to_owned())?;
        let mut name_bytes=vec![0;name_size];
        f.read_exact(&mut name_bytes).map_err(|_|"A GGUF tensor name is truncated.".to_owned())?;
        let name=String::from_utf8(name_bytes).map_err(|_|"A GGUF tensor name contains invalid UTF-8.".to_owned())?;
        tensor_name_bytes=tensor_name_bytes.checked_add(name.len() as u64)
            .ok_or_else(||"GGUF tensor-name byte count overflowed.".to_owned())?;
        if tensor_name_bytes>MAX_TENSOR_NAME_BYTES{
            return Err("GGUF tensor names exceed the aggregate safe inspection limit.".to_owned());
        }
        if !tensor_names.insert(name){return Err("The GGUF file contains duplicate tensor names.".to_owned());}
        let dimensions_count=read_u32(&mut f)?;
        if !(1..=4).contains(&dimensions_count){return Err("A GGUF tensor has an invalid dimension count.".to_owned());}
        let mut dimensions=[0u64;4];
        for dimension in dimensions.iter_mut().take(dimensions_count as usize){*dimension=read_u64(&mut f)?;}
        let type_code=read_u32(&mut f)?;let offset=read_u64(&mut f)?;
        if offset%u64::from(alignment)!=0{return Err("A GGUF tensor offset is not aligned as required by the file.".to_owned());}
        descriptors.push(TensorDescriptor{dimensions,dimensions_count:dimensions_count as u8,type_code,offset});
    }
    let descriptor_end=f.stream_position().map_err(|e|format!("Could not locate GGUF tensor data: {e}"))?;
    let data_start=align_up(descriptor_end,alignment)?;
    if data_start>file_size{return Err("GGUF tensor descriptors extend beyond the file.".to_owned());}
    let mut grouped:std::collections::BTreeMap<u32,TensorTypeSummary>=std::collections::BTreeMap::new();
    let mut known_total=0u64;let mut unknown_total=0u64;let mut analysed=0u64;
    let mut total_tensor_elements=0u64;
    // Descriptor order is not trusted. Sort by relative data offset so known payload
    // ranges can be checked for overlap without reading tensor weights.
    descriptors.sort_unstable_by_key(|tensor| tensor.offset);
    let mut previous_known_end:Option<u64>=None;
    let mut previous_descriptor_offset:Option<u64>=None;
    for tensor in descriptors{
        if previous_descriptor_offset==Some(tensor.offset){
            return Err("The GGUF file contains multiple tensor descriptors with the same data offset.".to_owned());
        }
        previous_descriptor_offset=Some(tensor.offset);
        let dimensions=&tensor.dimensions[..usize::from(tensor.dimensions_count)];
        let elements=tensor_element_count(dimensions)?;
        total_tensor_elements=total_tensor_elements.checked_add(elements)
            .ok_or_else(||"Aggregate GGUF tensor element count overflowed.".to_owned())?;
        let (name,block,block_bytes)=tensor_type_layout(tensor.type_code).unwrap_or(("UNKNOWN",0,0));
        let summary=grouped.entry(tensor.type_code).or_insert_with(||TensorTypeSummary{
            type_code:tensor.type_code,type_name:name.to_owned(),tensor_count:0,known_payload_bytes:0,unknown_size_tensor_count:0});
        summary.tensor_count=summary.tensor_count.checked_add(1).ok_or_else(||"Tensor count overflowed.".to_owned())?;
        let absolute=data_start.checked_add(tensor.offset).ok_or_else(||"GGUF tensor offset overflowed.".to_owned())?;
        // Even when a tensor type has an unknown payload layout, its start
        // must identify an actual byte in the file; EOF is not a valid data start.
        if absolute>=file_size{return Err("A GGUF tensor offset points beyond the file or to its end.".to_owned());}
        // The next descriptor's start is known even when its type has an unknown
        // payload size, so it must still not begin inside a preceding known payload.
        if previous_known_end.is_some_and(|previous_end| absolute<previous_end){
            return Err("A known GGUF tensor payload overlaps the next tensor offset.".to_owned());
        }
        if let Some(bytes)=tensor_payload_bytes(dimensions,block,block_bytes){
            let end=absolute.checked_add(bytes).ok_or_else(||"GGUF tensor payload size overflowed.".to_owned())?;
            if end>file_size{return Err("A GGUF tensor payload extends beyond the file.".to_owned());}
            previous_known_end=Some(end);
            summary.known_payload_bytes=summary.known_payload_bytes.checked_add(bytes).ok_or_else(||"Tensor payload total overflowed.".to_owned())?;
            known_total=known_total.checked_add(bytes).ok_or_else(||"Tensor payload total overflowed.".to_owned())?;
            analysed=analysed.checked_add(1).ok_or_else(||"Analysed tensor count overflowed.".to_owned())?;
        }else{
            summary.unknown_size_tensor_count=summary.unknown_size_tensor_count.checked_add(1).ok_or_else(||"Unknown tensor count overflowed.".to_owned())?;
            unknown_total=unknown_total.checked_add(1).ok_or_else(||"Unknown tensor count overflowed.".to_owned())?;
        }
    }
    let status=if unknown_total==0{"complete"}else if analysed==0{"unknown"}else{"partial"};
    let parameter_count_status=match reported_parameter_count{
        None=>"not_provided",
        Some(value) if value==total_tensor_elements=>"matches_reported_count",
        Some(value) if total_tensor_elements>value=>"tensor_elements_exceed_reported_count",
        Some(_)=>"tensor_elements_below_reported_count",
    };
    Ok(TensorStorageAnalysis{status:status.to_owned(),tensor_count,analysed_tensor_count:analysed,
        unknown_size_tensor_count:unknown_total,known_payload_bytes:known_total,tensor_element_count:total_tensor_elements,
        reported_parameter_count,parameter_count_status:parameter_count_status.to_owned(),data_start_offset:data_start,
        alignment,types:grouped.into_values().collect(),
        note:"Known payload sizes use verified GGML block layouts and exclude inter-tensor alignment padding. Unsupported types or dimensions that do not fit a known block layout remain unknown. This is stored tensor payload size, not RAM/VRAM required to run the model.".to_owned()})
}

pub fn inspect(input:&str)->Result<GgufMetadata,String>{
    let path=Path::new(input).canonicalize().map_err(|_|"The selected GGUF file could not be found.".to_owned())?;
    if !path.is_file(){return Err("The selected GGUF path is not a file.".to_owned());}
    if !path.extension().and_then(|v|v.to_str()).is_some_and(|v|v.eq_ignore_ascii_case("gguf")){return Err("Only .gguf files can be inspected.".to_owned());}
    let mut f=File::open(&path).map_err(|e|format!("Could not open the GGUF file: {e}"))?;
    // Keep the reported size tied to the same opened handle used by the parser.
    let file_size=f.metadata().map_err(|e|format!("Could not inspect the GGUF file: {e}"))?.len();
    let mut magic=[0;4];f.read_exact(&mut magic).map_err(|_|"The selected file is too small to be a GGUF file.".to_owned())?;
    if &magic!=b"GGUF"{return Err("The selected file does not have a valid GGUF header.".to_owned());}
    let version=read_u32(&mut f)?;if !(2..=3).contains(&version){return Err(format!("GGUF version {version} is not supported by this metadata inspector (supported: 2 and 3)."));}
    let tensors=read_u64(&mut f)?;let entries=read_u64(&mut f)?;
    let metadata_section_start=f.stream_position().map_err(|_|"Could not locate GGUF metadata section.".to_owned())?;
    if tensors>MAX_TENSORS{return Err("The GGUF file declares too many tensors for safe inspection.".to_owned());}
    if entries>MAX_METADATA_ENTRIES{return Err("The GGUF file declares too many metadata entries for safe inspection.".to_owned());}
    let(mut architecture,mut model_name,mut file_type_code,mut parameter_count,mut context_length,mut block_count,mut embedding_length,mut attention_head_count)=(None,None,None,None,None,None,None,None);
    let(mut attention_kv_head_count,mut attention_key_length,mut attention_value_length,mut rope_dimension_count,mut expert_count,mut expert_used_count,mut quantization_version,mut alignment)=(None,None,None,None,None,None,None,None);
    // Bound aggregate key storage instead of reserving from the untrusted entry count.
    let mut metadata_keys=HashSet::new();
    let mut metadata_key_bytes=0u64;
    let mut selected_fields=HashSet::new();
    let mut architecture_metadata_prefixes=HashSet::new();
    for _ in 0..entries{
        let remaining_key_budget=MAX_METADATA_KEY_BYTES.checked_sub(metadata_key_bytes)
            .ok_or_else(||"GGUF metadata key byte count exceeded its budget.".to_owned())?;
        let key=read_metadata_key(&mut f,remaining_key_budget)?;
        if key.is_empty(){return Err("The GGUF file contains an empty metadata key.".to_owned());}
        metadata_key_bytes=metadata_key_bytes.checked_add(key.len() as u64)
            .ok_or_else(||"GGUF metadata key byte count overflowed.".to_owned())?;
        if metadata_key_bytes>MAX_METADATA_KEY_BYTES{return Err("GGUF metadata keys exceed the aggregate safe inspection limit.".to_owned());}
        if !metadata_keys.insert(key.clone()){return Err("The GGUF file contains a duplicate metadata key.".to_owned());}
        let kind=read_u32(&mut f)?;
        let Some(field)=selected_metadata_field(&key) else{
            skip_value(&mut f,kind,metadata_section_start)?;
            enforce_metadata_section_limit(&mut f,metadata_section_start)?;
            continue;
        };
        if !selected_fields.insert(field){
            return Err(format!("The GGUF file contains multiple metadata keys for the same recognised field ({field}); the value is ambiguous."));
        }
        if let Some(prefix)=architecture_metadata_prefix(&key){architecture_metadata_prefixes.insert(prefix.to_owned());}
        let value=scalar(&mut f,kind)?;
        match key.as_str(){
            "general.architecture"=>architecture=Some(required_text(value,&key)?),
            "general.name"=>model_name=Some(required_text(value,&key)?),
            "general.file_type"=>file_type_code=Some(required_u32(value,&key)?),
            "general.parameter_count"=>parameter_count=Some(required_unsigned(value,&key)?),
            "general.quantization_version"=>quantization_version=Some(required_u32(value,&key)?),
            "general.alignment"=>alignment=Some(required_u32(value,&key)?),
            _ if key.ends_with(".context_length")=>context_length=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".block_count")=>block_count=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".embedding_length")=>embedding_length=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".attention.head_count_kv")=>attention_kv_head_count=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".attention.key_length")=>attention_key_length=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".attention.value_length")=>attention_value_length=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".attention.head_count")=>attention_head_count=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".rope.dimension_count")=>rope_dimension_count=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".expert_count")=>expert_count=Some(required_unsigned(value,&key)?),
            _ if key.ends_with(".expert_used_count")=>expert_used_count=Some(required_unsigned(value,&key)?),
            _=>{}
        }
    }
    if !architecture_metadata_prefixes.is_empty(){
        let declared=architecture.as_deref().ok_or_else(||"The GGUF file contains architecture-specific metadata but is missing general.architecture.".to_owned())?;
        if architecture_metadata_prefixes.iter().any(|prefix|prefix!=declared){
            return Err("The GGUF architecture-specific metadata namespace does not match general.architecture.".to_owned());
        }
    }
    Ok(GgufMetadata{format_version:version,file_size_bytes:file_size,tensor_count:tensors,metadata_entry_count:entries,architecture,model_name,file_type_code,file_type_description:file_type_code.and_then(file_type_description),parameter_count,context_length,block_count,embedding_length,attention_head_count,attention_kv_head_count,attention_key_length,attention_value_length,rope_dimension_count,expert_count,expert_used_count,quantization_version,alignment})
}
/// Conservative KV-cache estimate for the configured context, assuming FP16 (2-byte) K/V values.
/// This excludes model weights, allocator overhead, compute buffers, graph/workspace memory, and runtime-specific padding.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KvCacheEstimate {
    pub status: String, pub context_tokens: u64, pub bytes_per_element: u64,
    pub key_cache_bytes: Option<u64>, pub value_cache_bytes: Option<u64>, pub total_bytes: Option<u64>,
    pub reported_context_length: Option<u64>, pub context_limit_status: String, pub context_limit_note: String,
    pub assumptions: String,
}

fn context_limit_advisory(metadata: &GgufMetadata, context_tokens: u64) -> (String, String) {
    match metadata.context_length {
        Some(reported) if reported > 0 && context_tokens > reported => (
            "exceeds_reported_context".to_owned(),
            format!("The configured context ({context_tokens} tokens) exceeds the GGUF-reported context length ({reported} tokens). This is advisory only: some runtimes support RoPE scaling or context extension, and GGUF metadata is not a verified runtime limit.")
        ),
        Some(reported) if reported > 0 => (
            "within_reported_context".to_owned(),
            format!("The configured context ({context_tokens} tokens) is within the GGUF-reported context length ({reported} tokens). This does not prove that the runtime supports this context.")
        ),
        Some(_) => ("reported_context_unusable".to_owned(), "The GGUF file reports a zero context length, so it cannot provide a useful context comparison.".to_owned()),
        None => ("not_provided".to_owned(), "The GGUF file does not provide a context length for comparison.".to_owned()),
    }
}

pub fn estimate_kv_cache(metadata: &GgufMetadata, context_tokens: u64) -> Result<KvCacheEstimate, String> {
    if !(512..=32_768).contains(&context_tokens) { return Err("Context length must be between 512 and 32,768 tokens.".to_owned()); }
    const BYTES_PER_ELEMENT: u64 = 2;
    let (context_limit_status, context_limit_note) = context_limit_advisory(metadata, context_tokens);
    let (Some(layers), Some(kv_heads), Some(key_length), Some(value_length)) = (
        metadata.block_count, metadata.attention_kv_head_count, metadata.attention_key_length, metadata.attention_value_length,
    ) else {
        return Ok(KvCacheEstimate { status: "insufficient_metadata".to_owned(), context_tokens, bytes_per_element: BYTES_PER_ELEMENT,
            key_cache_bytes: None, value_cache_bytes: None, total_bytes: None,
            reported_context_length: metadata.context_length, context_limit_status, context_limit_note,
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
        reported_context_length: metadata.context_length, context_limit_status, context_limit_note,
        assumptions: "FP16 K/V cache (2 bytes per element), one sequence, using the configured context length and GGUF-reported dimensions. Actual runtime cache type, batching, parallel sequences, padding, and implementation overhead can change memory use.".to_owned() })
}

/// Evidence-only comparison of the estimated KV cache with a fresh host-RAM snapshot.
/// This deliberately excludes model weights and does not claim that the model will fit.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMemoryEvidence {
    pub status: String,
    pub architecture: Option<String>,
    pub model_file_size_bytes: u64,
    pub tensor_analysis_status: String,
    pub known_tensor_payload_bytes: u64,
    pub unknown_size_tensor_count: u64,
    pub context_tokens: u64,
    pub reported_context_length: Option<u64>,
    pub context_limit_status: String,
    pub context_limit_note: String,
    pub kv_cache_estimate_bytes: Option<u64>,
    pub total_system_memory_bytes: u64,
    pub available_system_memory_bytes: u64,
    pub kv_cache_percent_of_available_memory: Option<f64>,
    pub explanation: String,
    pub limitations: String,
}

pub fn assess_memory_evidence(
    metadata: &GgufMetadata,
    tensor_analysis: &TensorStorageAnalysis,
    total_memory_bytes: u64,
    available_memory_bytes: u64,
    context_tokens: u64,
) -> Result<ModelMemoryEvidence, String> {
    let estimate = estimate_kv_cache(metadata, context_tokens)?;
    let available = available_memory_bytes.min(total_memory_bytes);
    let kv_bytes = estimate.total_bytes;
    let (status, explanation, percentage) = match (kv_bytes, estimate.status.as_str(), total_memory_bytes) {
        (None, _, _) => (
            "insufficient_model_metadata",
            "The GGUF file does not report all dimensions needed for a KV-cache estimate. No missing values were guessed.",
            None,
        ),
        (Some(_), _, 0) => (
            "hardware_memory_unavailable",
            "The model's KV-cache estimate is available, but the operating system did not provide a usable total-RAM reading.",
            None,
        ),
        (Some(bytes), _, _) if bytes > available => (
            "kv_cache_exceeds_available_ram",
            "The estimated FP16 KV cache alone is larger than currently available system RAM. This is a strong host-memory warning, not a complete model-fit verdict.",
            if available > 0 { Some(bytes as f64 * 100.0 / available as f64) } else { None },
        ),
        (Some(bytes), _, _) => (
            "kv_cache_below_available_ram",
            "The estimated FP16 KV cache is below currently available system RAM. Model weights, runtime overhead, compute buffers, and other applications still need memory, so this does not mean the model will fit.",
            if available > 0 { Some(bytes as f64 * 100.0 / available as f64) } else { None },
        ),
    };
    Ok(ModelMemoryEvidence {
        status: status.to_owned(), architecture: metadata.architecture.clone(),
        model_file_size_bytes: metadata.file_size_bytes,
        tensor_analysis_status: tensor_analysis.status.clone(),
        known_tensor_payload_bytes: tensor_analysis.known_payload_bytes,
        unknown_size_tensor_count: tensor_analysis.unknown_size_tensor_count,
        context_tokens, reported_context_length: estimate.reported_context_length,
        context_limit_status: estimate.context_limit_status,
        context_limit_note: estimate.context_limit_note,
        kv_cache_estimate_bytes: kv_bytes,
        total_system_memory_bytes: total_memory_bytes,
        available_system_memory_bytes: available,
        kv_cache_percent_of_available_memory: percentage,
        explanation: explanation.to_owned(),
        limitations: "The KV estimate assumes FP16 cache values and one sequence. It excludes model-weight residency, runtime/allocator overhead, compute buffers, batching, parallel sequences, cache quantisation, GPU allocation, CPU/GPU offload, and concurrent workload changes. GGUF tensor payload totals are file-storage measurements, not RAM requirements. Hardware and GGUF metadata are snapshots and may be incomplete or inaccurate.".to_owned(),
    })
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
    fn metadata_keys_fixture(keys:&[&str])->Vec<u8>{
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&(keys.len() as u64).to_le_bytes());
        for (index,key) in keys.iter().enumerate(){
            push_string(&mut b,key);b.extend_from_slice(&4u32.to_le_bytes());
            b.extend_from_slice(&(32u32+(index as u32)*32).to_le_bytes());
        }
        let aligned=((b.len()+31)/32)*32;b.resize(aligned,0);b
    }
    fn metadata_bool_fixture(is_array:bool,value:u8)->Vec<u8>{
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&1u64.to_le_bytes());
        push_string(&mut b,"unknown.boolean");
        if is_array{
            b.extend_from_slice(&9u32.to_le_bytes());b.extend_from_slice(&7u32.to_le_bytes());
            b.extend_from_slice(&1u64.to_le_bytes());
        }else{b.extend_from_slice(&7u32.to_le_bytes());}
        b.push(value);
        let aligned=((b.len()+31)/32)*32;b.resize(aligned,0);b
    }
    fn temp(bytes:&[u8],ext:&str)->std::path::PathBuf{let p=std::env::temp_dir().join(format!("midsunday-gguf-{}-{}.{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed),ext));let mut f=File::create(&p).unwrap();f.write_all(bytes).unwrap();p}
    #[test]fn rejects_string_array_that_exceeds_metadata_section_budget_mid_entry(){
        let p=std::env::temp_dir().join(format!("midsunday-gguf-array-budget-{}-{}.gguf",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        let mut file=File::create(&p).unwrap();
        let start=MAX_METADATA_SECTION_BYTES-16;
        // Array header (12 bytes), string length (8 bytes), then the full payload.
        file.set_len(start+12+8+MAX_STRING_BYTES).unwrap();
        file.seek(SeekFrom::Start(start)).unwrap();
        file.write_all(&8u32.to_le_bytes()).unwrap();
        file.write_all(&1u64.to_le_bytes()).unwrap();
        file.write_all(&MAX_STRING_BYTES.to_le_bytes()).unwrap();
        file.write_all(&vec![b'x';MAX_STRING_BYTES as usize]).unwrap();
        drop(file);
        // File::create opens write-only; reopen read-only for the parser under test.
        let mut file=File::open(&p).unwrap();
        file.seek(SeekFrom::Start(start)).unwrap();
        let error=skip_value(&mut file,9,0).unwrap_err();
        assert!(error.contains("metadata section exceeds"));
        // The declared string is rejected immediately after its length field;
        // its payload is neither read nor allocated.
        assert_eq!(file.stream_position().unwrap(),start+12+8);
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_truncated_string_before_allocating_declared_payload(){
        let mut bytes=Vec::new();
        bytes.extend_from_slice(&1024u64.to_le_bytes());
        bytes.extend_from_slice(b"short");
        let p=temp(&bytes,"gguf");
        let mut file=File::open(&p).unwrap();
        let error=read_string(&mut file).unwrap_err();
        assert!(error.contains("truncated"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn reads_core_metadata_without_tensor_payloads(){let p=temp(&fixture(),"gguf");let m=inspect(p.to_str().unwrap()).unwrap();assert_eq!(m.format_version,3);assert_eq!(m.tensor_count,2);assert_eq!(m.architecture.as_deref(),Some("llama"));assert_eq!(m.model_name.as_deref(),Some("Tiny test model"));assert_eq!(m.file_type_code,Some(2));assert_eq!(m.file_type_description.as_deref(),Some("Q4_0"));assert_eq!(m.parameter_count,Some(7_000_000_000));assert_eq!(m.context_length,Some(8192));assert_eq!(m.block_count,Some(32));assert_eq!(m.embedding_length,Some(4096));assert_eq!(m.attention_head_count,Some(32));assert_eq!(m.attention_kv_head_count,Some(8));assert_eq!(m.attention_key_length,Some(128));assert_eq!(m.attention_value_length,Some(128));assert_eq!(m.rope_dimension_count,Some(128));assert_eq!(m.expert_count,Some(8));assert_eq!(m.expert_used_count,Some(2));assert_eq!(m.quantization_version,Some(2));assert_eq!(m.alignment,Some(32));let _=std::fs::remove_file(p);}
    #[test]fn rejects_metadata_key_payload_before_reading_past_aggregate_budget(){
        let p=std::env::temp_dir().join(format!("midsunday-gguf-key-preflight-{}-{}.gguf",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        let mut writer=File::create(&p).unwrap();
        writer.write_all(&u64::MAX.to_le_bytes()).unwrap();
        drop(writer);
        let mut reader=File::open(&p).unwrap();
        let error=read_metadata_key(&mut reader,0).unwrap_err();
        assert!(error.contains("metadata keys exceed the aggregate safe inspection limit"));
        assert_eq!(reader.stream_position().unwrap(),8);
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_aggregate_metadata_key_storage_limit_in_both_inspectors(){
        let long_key="k".repeat(MAX_STRING_BYTES as usize-1);
        let keys:Vec<String>=(0..9).map(|index|format!("{long_key}{index}")).collect();
        let borrowed:Vec<&str>=keys.iter().map(String::as_str).collect();
        let bytes=metadata_keys_fixture(&borrowed);
        let p=temp(&bytes,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("aggregate safe inspection limit"));
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("aggregate safe inspection limit"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_duplicate_metadata_keys_in_both_inspectors(){
        let duplicate=metadata_keys_fixture(&["general.alignment","general.alignment"]);
        let p=temp(&duplicate,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("duplicate metadata key"));
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("duplicate metadata key"));
        let _=std::fs::remove_file(p);

        let distinct=metadata_keys_fixture(&["unknown.first","unknown.second"]);
        let p=temp(&distinct,"gguf");
        assert_eq!(inspect(p.to_str().unwrap()).unwrap().metadata_entry_count,2);
        assert_eq!(analyse_tensor_storage(p.to_str().unwrap()).unwrap().tensor_count,0);
        let _=std::fs::remove_file(p);
    }
    #[test]fn validates_metadata_keys_and_boolean_encodings(){
        let empty=metadata_keys_fixture(&[""]);
        let p=temp(&empty,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("empty metadata key"));
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("empty metadata key"));
        let _=std::fs::remove_file(p);

        for is_array in [false,true]{
            let valid=metadata_bool_fixture(is_array,1);
            let p=temp(&valid,"gguf");
            assert!(inspect(p.to_str().unwrap()).is_ok());
            assert!(analyse_tensor_storage(p.to_str().unwrap()).is_ok());
            let _=std::fs::remove_file(p);

            let invalid=metadata_bool_fixture(is_array,2);
            let p=temp(&invalid,"gguf");
            assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("must be encoded as 0 or 1"));
            assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("must be encoded as 0 or 1"));
            let _=std::fs::remove_file(p);
        }
    }
    #[test]fn rejects_invalid_magic_and_versions(){let p=temp(b"NOPE","gguf");assert!(inspect(p.to_str().unwrap()).is_err());let _=std::fs::remove_file(p);let mut b=fixture();b[4..8].copy_from_slice(&99u32.to_le_bytes());let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("version 99"));let _=std::fs::remove_file(p);}
    #[test]fn rejects_excessive_tensor_count_in_both_inspectors(){
        let mut b=fixture();b[8..16].copy_from_slice(&(MAX_TENSORS+1).to_le_bytes());
        let p=temp(&b,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("too many tensors"));
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("too many tensors"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_aggregate_tensor_descriptor_memory_budget_before_allocation(){
        let count=MAX_TENSOR_DESCRIPTOR_MEMORY_BYTES/ESTIMATED_TENSOR_DESCRIPTOR_BYTES+1;
        let mut bytes=b"GGUF".to_vec();bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&count.to_le_bytes());bytes.extend_from_slice(&0u64.to_le_bytes());
        let p=temp(&bytes,"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("aggregate safe memory budget"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn accepts_tensor_descriptor_count_within_memory_budget_before_descriptor_validation(){
        let count=MAX_TENSOR_DESCRIPTOR_MEMORY_BYTES/ESTIMATED_TENSOR_DESCRIPTOR_BYTES;
        let mut bytes=b"GGUF".to_vec();bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&count.to_le_bytes());bytes.extend_from_slice(&0u64.to_le_bytes());
        let p=temp(&bytes,"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("cannot fit within the remaining file bytes"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_impossible_tensor_descriptor_count_before_allocation(){
        let mut b=metadata_keys_fixture(&[]);
        b[8..16].copy_from_slice(&1u64.to_le_bytes());
        let p=temp(&b,"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("cannot fit within the remaining file bytes"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_truncated_metadata_and_unreasonable_counts(){let mut b=fixture();b.truncate(35);let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).is_err());let _=std::fs::remove_file(p);let mut b=fixture();b[20..28].copy_from_slice(&(MAX_METADATA_ENTRIES+1).to_le_bytes());let p=temp(&b,"gguf");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("too many metadata"));let _=std::fs::remove_file(p);}
    #[test]fn skips_unknown_string_and_array_metadata(){let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&2u64.to_le_bytes());push_string(&mut b,"unknown.string");b.extend_from_slice(&8u32.to_le_bytes());push_string(&mut b,"ignored");push_string(&mut b,"unknown.array");b.extend_from_slice(&9u32.to_le_bytes());b.extend_from_slice(&4u32.to_le_bytes());b.extend_from_slice(&3u64.to_le_bytes());for n in [1u32,2,3]{b.extend_from_slice(&n.to_le_bytes());}let p=temp(&b,"gguf");assert_eq!(inspect(p.to_str().unwrap()).unwrap().tensor_count,0);let _=std::fs::remove_file(p);}
    fn single_metadata_fixture(key:&str,kind:u32,value:&[u8])->Vec<u8>{
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&1u64.to_le_bytes());
        push_string(&mut b,key);b.extend_from_slice(&kind.to_le_bytes());b.extend_from_slice(value);
        while b.len()%32!=0{b.push(0);}b
    }
    #[test]fn rejects_ambiguous_architecture_metadata_aliases(){
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&2u64.to_le_bytes());
        for key in ["llama.block_count","mistral.block_count"]{
            push_string(&mut b,key);b.extend_from_slice(&10u32.to_le_bytes());
            b.extend_from_slice(&32u64.to_le_bytes());
        }
        while b.len()%32!=0{b.push(0);}
        let p=temp(&b,"gguf");
        let error=inspect(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("multiple metadata keys"));
        assert!(error.contains("block_count"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_architecture_metadata_namespace_mismatch_when_architecture_appears_later(){
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&2u64.to_le_bytes());
        push_string(&mut b,"llama.block_count");b.extend_from_slice(&10u32.to_le_bytes());b.extend_from_slice(&32u64.to_le_bytes());
        push_string(&mut b,"general.architecture");b.extend_from_slice(&8u32.to_le_bytes());push_string(&mut b,"mistral");
        while b.len()%32!=0{b.push(0);}
        let p=temp(&b,"gguf");let error=inspect(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("namespace does not match"));let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_architecture_metadata_without_declared_architecture(){
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&1u64.to_le_bytes());
        push_string(&mut b,"llama.block_count");b.extend_from_slice(&10u32.to_le_bytes());b.extend_from_slice(&32u64.to_le_bytes());
        while b.len()%32!=0{b.push(0);}
        let p=temp(&b,"gguf");let error=inspect(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("missing general.architecture"));let _=std::fs::remove_file(p);
    }
    #[test]fn ignores_unrelated_namespaced_metadata_without_suffix_collisions(){
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&0u64.to_le_bytes());b.extend_from_slice(&2u64.to_le_bytes());
        for key in ["vendor_a.custom_value","vendor_b.custom_value"]{
            push_string(&mut b,key);b.extend_from_slice(&10u32.to_le_bytes());
            b.extend_from_slice(&32u64.to_le_bytes());
        }
        while b.len()%32!=0{b.push(0);}
        let p=temp(&b,"gguf");
        assert!(inspect(p.to_str().unwrap()).is_ok());
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_wrong_types_and_out_of_range_selected_metadata(){
        let mut string_value=Vec::new();push_string(&mut string_value,"seven");
        let wrong_string=single_metadata_fixture("general.parameter_count",8,&string_value);
        let p=temp(&wrong_string,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("non-negative integer"));
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("non-negative integer"));
        let _=std::fs::remove_file(p);

        let mut alignment_string=Vec::new();push_string(&mut alignment_string,"32");
        let wrong_alignment=single_metadata_fixture("general.alignment",8,&alignment_string);
        let p=temp(&wrong_alignment,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("non-negative integer"));
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("non-negative integer"));
        let _=std::fs::remove_file(p);

        let wrong_architecture=single_metadata_fixture("general.architecture",4,&32u32.to_le_bytes());
        let p=temp(&wrong_architecture,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("must contain a string"));
        let _=std::fs::remove_file(p);

        let out_of_range=single_metadata_fixture("general.file_type",10,&u64::MAX.to_le_bytes());
        let p=temp(&out_of_range,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("32-bit range"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn skips_fixed_width_arrays_in_bounded_bulk_and_rejects_truncation(){
        let mut valid=b"GGUF".to_vec();valid.extend_from_slice(&3u32.to_le_bytes());
        valid.extend_from_slice(&0u64.to_le_bytes());valid.extend_from_slice(&1u64.to_le_bytes());
        push_string(&mut valid,"unknown.numeric_array");
        valid.extend_from_slice(&9u32.to_le_bytes());valid.extend_from_slice(&4u32.to_le_bytes());
        valid.extend_from_slice(&3u64.to_le_bytes());valid.extend_from_slice(&[1u8,0,0,0,2,0,0,0,3,0,0,0]);
        while valid.len()%32!=0{valid.push(0);}
        let p=temp(&valid,"gguf");
        assert!(inspect(p.to_str().unwrap()).is_ok());
        assert!(analyse_tensor_storage(p.to_str().unwrap()).is_ok());
        let _=std::fs::remove_file(p);

        let mut truncated=b"GGUF".to_vec();truncated.extend_from_slice(&3u32.to_le_bytes());
        truncated.extend_from_slice(&0u64.to_le_bytes());truncated.extend_from_slice(&1u64.to_le_bytes());
        push_string(&mut truncated,"unknown.numeric_array");
        truncated.extend_from_slice(&9u32.to_le_bytes());truncated.extend_from_slice(&4u32.to_le_bytes());
        truncated.extend_from_slice(&3u64.to_le_bytes());truncated.extend_from_slice(&[1u8,0,0,0,2,0,0,0]);
        let p=temp(&truncated,"gguf");
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("truncated"));
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("truncated"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn estimates_fp16_kv_cache_from_architecture_dimensions(){
        let p=temp(&fixture(), "gguf"); let metadata=inspect(p.to_str().unwrap()).unwrap();
        let estimate=estimate_kv_cache(&metadata, 8192).unwrap();
        assert_eq!(estimate.status, "estimated");
        assert_eq!(estimate.key_cache_bytes, Some(536_870_912));
        assert_eq!(estimate.value_cache_bytes, Some(536_870_912));
        assert_eq!(estimate.total_bytes, Some(1_073_741_824));
        let _=std::fs::remove_file(p);
    }
    #[test]fn reports_context_length_advisory_without_rejecting_estimate(){
        let p=temp(&fixture(), "gguf"); let metadata=inspect(p.to_str().unwrap()).unwrap();
        let within=estimate_kv_cache(&metadata, 8192).unwrap();
        assert_eq!(within.context_limit_status, "within_reported_context");
        assert_eq!(within.reported_context_length, Some(8192));
        let above=estimate_kv_cache(&metadata, 16384).unwrap();
        assert_eq!(above.status, "estimated");
        assert_eq!(above.context_limit_status, "exceeds_reported_context");
        assert!(above.context_limit_note.contains("RoPE scaling"));
        let mut missing=metadata.clone(); missing.context_length=None;
        assert_eq!(estimate_kv_cache(&missing, 4096).unwrap().context_limit_status, "not_provided");
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
    #[test]fn memory_evidence_warns_when_kv_cache_alone_exceeds_available_ram(){
        let p=temp(&fixture(), "gguf"); let metadata=inspect(p.to_str().unwrap()).unwrap();
        let tensor=TensorStorageAnalysis{status:"complete".to_owned(),tensor_count:2,analysed_tensor_count:2,unknown_size_tensor_count:0,known_payload_bytes:123,tensor_element_count:0,reported_parameter_count:None,parameter_count_status:"not_provided".to_owned(),data_start_offset:0,alignment:32,types:Vec::new(),note:String::new()};
        let report=assess_memory_evidence(&metadata,&tensor,8*1024*1024*1024,512*1024*1024,8192).unwrap();
        assert_eq!(report.status,"kv_cache_exceeds_available_ram");
        assert_eq!(report.kv_cache_estimate_bytes,Some(1024*1024*1024));
        assert_eq!(report.kv_cache_percent_of_available_memory,Some(200.0));
        assert_eq!(report.known_tensor_payload_bytes,123);
        let _=std::fs::remove_file(p);
    }
    #[test]fn memory_evidence_never_calls_kv_below_available_ram_a_fit_guarantee(){
        let p=temp(&fixture(), "gguf"); let metadata=inspect(p.to_str().unwrap()).unwrap();
        let tensor=TensorStorageAnalysis{status:"partial".to_owned(),tensor_count:2,analysed_tensor_count:1,unknown_size_tensor_count:1,known_payload_bytes:123,tensor_element_count:0,reported_parameter_count:None,parameter_count_status:"not_provided".to_owned(),data_start_offset:0,alignment:32,types:Vec::new(),note:String::new()};
        let report=assess_memory_evidence(&metadata,&tensor,16*1024*1024*1024,8*1024*1024*1024,8192).unwrap();
        assert_eq!(report.status,"kv_cache_below_available_ram");
        assert!(report.limitations.contains("model-weight residency"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn memory_evidence_reports_missing_metadata_and_unavailable_hardware(){
        let p=temp(&fixture(), "gguf"); let mut metadata=inspect(p.to_str().unwrap()).unwrap();
        let tensor=TensorStorageAnalysis{status:"unknown".to_owned(),tensor_count:2,analysed_tensor_count:0,unknown_size_tensor_count:2,known_payload_bytes:0,tensor_element_count:0,reported_parameter_count:None,parameter_count_status:"not_provided".to_owned(),data_start_offset:0,alignment:32,types:Vec::new(),note:String::new()};
        metadata.attention_key_length=None;
        assert_eq!(assess_memory_evidence(&metadata,&tensor,8,4,2048).unwrap().status,"insufficient_model_metadata");
        metadata.attention_key_length=Some(128);
        assert_eq!(assess_memory_evidence(&metadata,&tensor,0,0,2048).unwrap().status,"hardware_memory_unavailable");
        let _=std::fs::remove_file(p);
    }
        fn tensor_fixture_many(tensors:&[(u32,u64,u64)],payload:&[u8])->Vec<u8>{
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&(tensors.len() as u64).to_le_bytes());b.extend_from_slice(&1u64.to_le_bytes());
        entry_u32(&mut b,"general.alignment",32);
        for (index,(type_code,dimension,offset)) in tensors.iter().enumerate(){
            push_string(&mut b,&format!("test.tensor.{index}"));b.extend_from_slice(&1u32.to_le_bytes());
            b.extend_from_slice(&dimension.to_le_bytes());b.extend_from_slice(&type_code.to_le_bytes());
            b.extend_from_slice(&offset.to_le_bytes());
        }
        let aligned=((b.len()+31)/32)*32;b.resize(aligned,0);
        b.extend_from_slice(payload);b
    }
    fn tensor_fixture_with_names(names:&[&[u8]])->Vec<u8>{
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&(names.len() as u64).to_le_bytes());b.extend_from_slice(&1u64.to_le_bytes());
        entry_u32(&mut b,"general.alignment",32);
        for (index,name) in names.iter().enumerate(){
            b.extend_from_slice(&(name.len() as u64).to_le_bytes());b.extend_from_slice(name);
            b.extend_from_slice(&1u32.to_le_bytes());b.extend_from_slice(&1u64.to_le_bytes());
            b.extend_from_slice(&0u32.to_le_bytes());b.extend_from_slice(&((index as u64)*32).to_le_bytes());
        }
        let aligned=((b.len()+31)/32)*32;b.resize(aligned,0);b.extend_from_slice(&[0;64]);b
    }
    fn tensor_fixture(type_code:u32,dimension:u64,offset:u64,payload:&[u8])->Vec<u8>{
        tensor_fixture_many(&[(type_code,dimension,offset)],payload)
    }
    fn tensor_fixture_dimensions(dimensions:&[u64],type_code:u32,reported_count:Option<u64>,payload:&[u8])->Vec<u8>{
        let mut b=b"GGUF".to_vec();b.extend_from_slice(&3u32.to_le_bytes());
        b.extend_from_slice(&1u64.to_le_bytes());
        b.extend_from_slice(&(if reported_count.is_some(){2u64}else{1u64}).to_le_bytes());
        entry_u32(&mut b,"general.alignment",32);
        if let Some(value)=reported_count{
            push_string(&mut b,"general.parameter_count");b.extend_from_slice(&10u32.to_le_bytes());b.extend_from_slice(&value.to_le_bytes());
        }
        push_string(&mut b,"test.tensor");b.extend_from_slice(&(dimensions.len() as u32).to_le_bytes());
        for dimension in dimensions{b.extend_from_slice(&dimension.to_le_bytes());}
        b.extend_from_slice(&type_code.to_le_bytes());b.extend_from_slice(&0u64.to_le_bytes());
        let aligned=((b.len()+31)/32)*32;b.resize(aligned,0);b.extend_from_slice(payload);b
    }
    #[test]fn rejects_aggregate_tensor_name_storage_limit_before_retaining_more_names(){
        let mut bytes=b"GGUF".to_vec();bytes.extend_from_slice(&3u32.to_le_bytes());
        let count=(MAX_TENSOR_NAME_BYTES/64+1) as u64;
        bytes.extend_from_slice(&count.to_le_bytes());bytes.extend_from_slice(&0u64.to_le_bytes());
        for index in 0..count{
            push_string(&mut bytes,&format!("{index:064}"));
            bytes.extend_from_slice(&1u32.to_le_bytes());bytes.extend_from_slice(&1u64.to_le_bytes());
            bytes.extend_from_slice(&99u32.to_le_bytes());bytes.extend_from_slice(&0u64.to_le_bytes());
        }
        let aligned=((bytes.len()+31)/32)*32;bytes.resize(aligned,0);
        let p=temp(&bytes,"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("aggregate safe inspection limit"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn analyses_known_q4_0_tensor_payload_without_reading_weights(){
        let p=temp(&tensor_fixture(2,32,0,&[0;18]),"gguf");
        let report=analyse_tensor_storage(p.to_str().unwrap()).unwrap();
        assert_eq!(report.status,"complete");assert_eq!(report.tensor_count,1);
        assert_eq!(report.analysed_tensor_count,1);assert_eq!(report.unknown_size_tensor_count,0);
        assert_eq!(report.known_payload_bytes,18);assert_eq!(report.types[0].type_name,"Q4_0");
        assert_eq!(report.types[0].known_payload_bytes,18);
        let _=std::fs::remove_file(p);
    }
    #[test]fn keeps_unsupported_tensor_types_unknown(){
        let p=temp(&tensor_fixture(99,32,0,&[0;18]),"gguf");
        let report=analyse_tensor_storage(p.to_str().unwrap()).unwrap();
        assert_eq!(report.status,"unknown");assert_eq!(report.analysed_tensor_count,0);
        assert_eq!(report.unknown_size_tensor_count,1);assert_eq!(report.known_payload_bytes,0);
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_tensor_payloads_outside_file_and_bad_offsets(){
        let p=temp(&tensor_fixture(2,32,32,&[0;18]),"gguf");
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("beyond the file"));
        let _=std::fs::remove_file(p);
        let p=temp(&tensor_fixture(2,32,1,&[0;18]),"gguf");
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("not aligned"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_unknown_tensor_offsets_at_or_beyond_eof(){
        let p=temp(&tensor_fixture(99,32,0,&[]),"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("beyond the file or to its end"));
        let _=std::fs::remove_file(p);

        let p=temp(&tensor_fixture(99,32,32,&[]),"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("beyond the file or to its end"));
        let _=std::fs::remove_file(p);

        let p=temp(&tensor_fixture(99,32,0,&[0;1]),"gguf");
        let report=analyse_tensor_storage(p.to_str().unwrap()).unwrap();
        assert_eq!(report.unknown_size_tensor_count,1);
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_duplicate_tensor_data_offsets_even_for_unknown_types(){
        let p=temp(&tensor_fixture_many(&[(999,32,0),(0,1,0)],&[0;64]),"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("same data offset"));let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_overlapping_known_tensor_payload_ranges(){
        let p=temp(&tensor_fixture_many(&[(2,64,0),(2,32,32)],&[0;64]),"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("overlap"));let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_known_payload_overlapping_unknown_tensor_start(){
        // F32[16] occupies 64 bytes; the unsupported tensor begins 32 bytes later.
        let p=temp(&tensor_fixture_many(&[(0,16,0),(99,32,32)],&[0;70]),"gguf");
        let error=analyse_tensor_storage(p.to_str().unwrap()).unwrap_err();
        assert!(error.contains("overlaps the next tensor offset"));let _=std::fs::remove_file(p);
    }
    #[test]fn accepts_non_overlapping_known_tensor_payload_ranges_in_offset_order(){
        let p=temp(&tensor_fixture_many(&[(2,32,32),(2,32,0)],&[0;50]),"gguf");
        let report=analyse_tensor_storage(p.to_str().unwrap()).unwrap();
        assert_eq!(report.status,"complete");assert_eq!(report.analysed_tensor_count,2);
        assert_eq!(report.known_payload_bytes,36);let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_tensor_dimensions_that_break_known_block_layout(){
        let p=temp(&tensor_fixture(2,31,0,&[0;18]),"gguf");
        let report=analyse_tensor_storage(p.to_str().unwrap()).unwrap();
        assert_eq!(report.status,"unknown");assert_eq!(report.unknown_size_tensor_count,1);
        let _=std::fs::remove_file(p);
    }
    #[test]fn aggregates_tensor_elements_and_compares_reported_parameter_count(){
        let p=temp(&tensor_fixture_dimensions(&[4,8],0,Some(32),&[0;128]),"gguf");
        let report=analyse_tensor_storage(p.to_str().unwrap()).unwrap();
        assert_eq!(report.tensor_element_count,32);
        assert_eq!(report.reported_parameter_count,Some(32));
        assert_eq!(report.parameter_count_status,"matches_reported_count");
        let _=std::fs::remove_file(p);

        let p=temp(&tensor_fixture_dimensions(&[4,8],99,Some(40),&[0;1]),"gguf");
        let report=analyse_tensor_storage(p.to_str().unwrap()).unwrap();
        assert_eq!(report.tensor_element_count,32);
        assert_eq!(report.parameter_count_status,"tensor_elements_below_reported_count");
        assert_eq!(report.unknown_size_tensor_count,1);
        let _=std::fs::remove_file(p);
    }
    #[test]fn reports_missing_parameter_count_and_rejects_bad_tensor_dimensions(){
        let p=temp(&tensor_fixture(0,16,0,&[0;64]),"gguf");
        let report=analyse_tensor_storage(p.to_str().unwrap()).unwrap();
        assert_eq!(report.tensor_element_count,16);
        assert_eq!(report.reported_parameter_count,None);
        assert_eq!(report.parameter_count_status,"not_provided");
        let _=std::fs::remove_file(p);

        let p=temp(&tensor_fixture_dimensions(&[0,2],0,None,&[]),"gguf");
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("zero-sized"));
        let _=std::fs::remove_file(p);

        let p=temp(&tensor_fixture_dimensions(&[u64::MAX,2],0,None,&[]),"gguf");
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("element count overflowed"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn validates_tensor_descriptor_names(){
        let p=temp(&tensor_fixture_with_names(&[b""]), "gguf");
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("between 1 and 64"));
        let _=std::fs::remove_file(p);

        let p=temp(&tensor_fixture_with_names(&[&[0xff]]), "gguf");
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("invalid UTF-8"));
        let _=std::fs::remove_file(p);

        let p=temp(&tensor_fixture_with_names(&[b"duplicate",b"duplicate"]), "gguf");
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("duplicate tensor names"));
        let _=std::fs::remove_file(p);

        let p=temp(&tensor_fixture_with_names(&[b"layer.0.weight",b"layer.1.weight"]), "gguf");
        assert_eq!(analyse_tensor_storage(p.to_str().unwrap()).unwrap().tensor_count,2);
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_aggregate_metadata_section_budget_in_both_inspectors(){
        let p=std::env::temp_dir().join(format!("midsunday-gguf-large-metadata-{}-{}.gguf",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        let mut f=File::create(&p).unwrap();
        f.write_all(b"GGUF").unwrap();f.write_all(&3u32.to_le_bytes()).unwrap();
        f.write_all(&0u64.to_le_bytes()).unwrap();f.write_all(&1u64.to_le_bytes()).unwrap();
        let mut key=Vec::new();push_string(&mut key,"vendor.large_array");f.write_all(&key).unwrap();
        f.write_all(&9u32.to_le_bytes()).unwrap();f.write_all(&10u32.to_le_bytes()).unwrap();
        // One valid u64 array exceeds the aggregate section budget while staying
        // below the independent ten-million-item array limit.
        let count=MAX_METADATA_SECTION_BYTES/8+1;f.write_all(&count.to_le_bytes()).unwrap();
        let payload=count.checked_mul(8).unwrap();
        let end=f.metadata().unwrap().len().checked_add(payload).unwrap();f.set_len(end).unwrap();drop(f);
        assert!(inspect(p.to_str().unwrap()).unwrap_err().contains("aggregate safe inspection limit"));
        assert!(analyse_tensor_storage(p.to_str().unwrap()).unwrap_err().contains("aggregate safe inspection limit"));
        let _=std::fs::remove_file(p);
    }
    #[test]fn rejects_non_gguf_extensions(){let p=temp(&fixture(),"bin");assert!(inspect(p.to_str().unwrap()).unwrap_err().contains(".gguf"));let _=std::fs::remove_file(p);}
}