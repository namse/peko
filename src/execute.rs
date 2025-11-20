use crate::wasm_code_provider::{self, WasmCodeProvider};
use sonic_rs::JsonValueTrait;
use wasmtime::{
    component::{Component, StreamReader, Type, Val},
    *,
};

pub async fn execute<Wcp: WasmCodeProvider>(
    engine: Engine,
    wasm_code_provider: Wcp,
    code_id: &str,
    fn_name: &str,
    body_bytes: &[u8],
) -> Result<(), Error> {
    let wasm_code = wasm_code_provider.get_wasm_code(code_id).await?;
    let component = Component::new(&engine, wasm_code)?;
    let linker = component::Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &component)?;

    let Some(func) = instance.get_func(&mut store, fn_name) else {
        return Err(Error::FuncNotFound);
    };

    let params = convert_body_to_vals(func.params(&store), body_bytes)?;
    let mut result = [Val::String("".into())];

    func.call(&mut store, &params, &mut result)?;

    todo!()
}

fn convert_body_to_vals(
    types: Box<[(String, Type)]>,
    body_bytes: &[u8],
) -> Result<Vec<Val>, Error> {
    let mut body: sonic_rs::Object =
        sonic_rs::from_slice(body_bytes).map_err(|_| Error::ParsingBodyFailed)?;

    let mut vals = Vec::with_capacity(types.len());

    for (name, ty) in types {
        let json_val = body.remove(&name).ok_or(Error::ParsingBodyFailed)?;
        vals.push(convert_json_value_to_wit_val(json_val, &ty)?);
    }

    Ok(vals)
}

fn convert_json_value_to_wit_val(json_val: sonic_rs::Value, ty: &Type) -> Result<Val, Error> {
    Ok(match ty {
        Type::Bool => Val::Bool(json_val.as_bool().ok_or(Error::ParsingBodyFailed)?),
        Type::S8 => Val::S8(json_val.as_u64().ok_or(Error::ParsingBodyFailed)? as i8),
        Type::U8 => Val::U8(json_val.as_u64().ok_or(Error::ParsingBodyFailed)? as u8),
        Type::S16 => Val::S16(json_val.as_u64().ok_or(Error::ParsingBodyFailed)? as i16),
        Type::U16 => Val::U16(json_val.as_u64().ok_or(Error::ParsingBodyFailed)? as u16),
        Type::S32 => Val::S32(json_val.as_u64().ok_or(Error::ParsingBodyFailed)? as i32),
        Type::U32 => Val::U32(json_val.as_u64().ok_or(Error::ParsingBodyFailed)? as u32),
        Type::S64 => Val::S64(json_val.as_u64().ok_or(Error::ParsingBodyFailed)? as i64),
        Type::U64 => Val::U64(json_val.as_u64().ok_or(Error::ParsingBodyFailed)?),
        Type::Float32 => Val::Float32(json_val.as_f64().ok_or(Error::ParsingBodyFailed)? as f32),
        Type::Float64 => Val::Float64(json_val.as_f64().ok_or(Error::ParsingBodyFailed)?),
        Type::Char => Val::Char(
            json_val
                .as_str()
                .and_then(|str| str.chars().next())
                .ok_or(Error::ParsingBodyFailed)?,
        ),
        Type::String => Val::String(
            json_val
                .as_str()
                .ok_or(Error::ParsingBodyFailed)?
                .to_string(),
        ),
        Type::List(list) => {
            let array = json_val.into_array().ok_or(Error::ParsingBodyFailed)?;
            let item_type = list.ty();

            let mut list = Vec::with_capacity(array.len());

            for val in array {
                let item = convert_json_value_to_wit_val(val, &item_type)?;
                list.push(item);
            }

            Val::List(list)
        }
        Type::Record(record) => {
            let mut object = json_val.into_object().ok_or(Error::ParsingBodyFailed)?;
            let mut fields = Vec::with_capacity(record.fields().len());

            for field in record.fields() {
                let val = convert_json_value_to_wit_val(
                    object.remove(&field.name).ok_or(Error::ParsingBodyFailed)?,
                    &field.ty,
                )?;
                fields.push((field.name.to_string(), val));
            }

            Val::Record(fields)
        }
        Type::Tuple(tuple) => {
            let array = json_val.into_array().ok_or(Error::ParsingBodyFailed)?;
            let mut items = Vec::with_capacity(tuple.types().len());
            if array.len() != tuple.types().len() {
                return Err(Error::ParsingBodyFailed);
            }

            for (val, ty) in array.into_iter().zip(tuple.types()) {
                let item = convert_json_value_to_wit_val(val, &ty)?;
                items.push(item);
            }

            Val::Tuple(items)
        }
        Type::Variant(variant) => {
            let mut array = json_val.into_array().ok_or(Error::ParsingBodyFailed)?;
            if array.is_empty() {
                return Err(Error::ParsingBodyFailed);
            }
            let variant_name = array
                .first()
                .ok_or(Error::ParsingBodyFailed)?
                .as_str()
                .ok_or(Error::ParsingBodyFailed)?
                .to_string();

            let case = variant
                .cases()
                .find(|case| case.name == variant_name)
                .ok_or(Error::ParsingBodyFailed)?;

            let Some(ty) = case.ty else {
                if array.len() != 1 {
                    return Err(Error::ParsingBodyFailed);
                }
                return Ok(Val::Variant(variant_name.to_string(), None));
            };

            if array.len() != 2 {
                return Err(Error::ParsingBodyFailed);
            }

            let variant_json_data = array.swap_remove(1);

            let variant_val = convert_json_value_to_wit_val(variant_json_data, &ty)?;

            Val::Variant(variant_name.to_string(), Some(Box::new(variant_val)))
        }
        Type::Enum(eenum) => {
            let string = json_val.as_str().ok_or(Error::ParsingBodyFailed)?;
            if !eenum.names().any(|name| name == string) {
                return Err(Error::ParsingBodyFailed);
            }
            Val::Enum(string.to_string())
        }
        Type::Option(option_type) => {
            let ty = option_type.ty();
            if json_val.is_null() {
                return Ok(Val::Option(None));
            }
            let val = convert_json_value_to_wit_val(json_val, &ty)?;
            Val::Option(Some(Box::new(val)))
        }
        Type::Result(result_type) => {
            let mut object = json_val.into_object().ok_or(Error::ParsingBodyFailed)?;
            if let Some(val) = object.remove(&"ok") {
                match result_type.ok() {
                    Some(ok_ty) => {
                        let val = convert_json_value_to_wit_val(val, &ok_ty)?;
                        return Ok(Val::Result(Ok(Some(Box::new(val)))));
                    }
                    None => return Ok(Val::Result(Ok(None))),
                };
            }
            if let Some(val) = object.remove(&"err") {
                match result_type.err() {
                    Some(err_ty) => {
                        let val = convert_json_value_to_wit_val(val, &err_ty)?;
                        return Ok(Val::Result(Err(Some(Box::new(val)))));
                    }
                    None => return Ok(Val::Result(Err(None))),
                };
            }
            return Err(Error::ParsingBodyFailed);
        }
        Type::Flags(flags) => todo!(),
        Type::Own(resource_type) => todo!(),
        Type::Borrow(resource_type) => todo!(),
        Type::Future(future_type) => todo!(),
        Type::Stream(stream_type) => todo!(),
        Type::ErrorContext => todo!(),
    })
}

pub enum Error {
    WasmCodeProvider(wasm_code_provider::Error),
    Wasmtime(wasmtime::Error),
    FuncNotFound,
    ParsingBodyFailed,
}
impl From<wasm_code_provider::Error> for Error {
    fn from(value: wasm_code_provider::Error) -> Self {
        Self::WasmCodeProvider(value)
    }
}
impl From<wasmtime::Error> for Error {
    fn from(value: wasmtime::Error) -> Self {
        Self::Wasmtime(value)
    }
}
