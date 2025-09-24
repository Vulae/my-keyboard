use std::collections::HashMap;

use regex::Regex;

use super::QueryError;

// https://unix.stackexchange.com/questions/74903/explain-ev-in-proc-bus-input-devices-data

#[derive(Debug, Clone)]
pub struct UnparsedQueryDeviceProperty {
    pub ident: char,
    pub content: String,
}

impl UnparsedQueryDeviceProperty {
    fn parse(str: &str) -> Result<Self, QueryError> {
        let Some((_, [ident_str, content])) = Regex::new(r"^([a-zA-Z]): ?(.*)$")
            .unwrap()
            .captures(str)
            .map(|i| i.extract())
        else {
            return Err(QueryError::Malformed("Malformed device property"));
        };
        let ident_chars = ident_str.chars().collect::<Vec<_>>();
        if ident_chars.len() != 1 {
            return Err(QueryError::Malformed("Malformed device property"));
        }
        Ok(Self {
            ident: ident_chars[0],
            content: content.to_owned(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct QueryDevice {
    pub id_bus_type: u16,
    pub id_vendor: u16,
    pub id_product: u16,
    pub id_version: u16,

    pub name: String,
    pub physical_path: String,
    pub sys_path: String,
    pub handlers: Box<[String]>,
    pub bitmaps: HashMap<String, Box<[u64]>>,

    pub unparsed_properties: Box<[UnparsedQueryDeviceProperty]>,
}

impl QueryDevice {
    fn parse(str: &str) -> Result<Self, QueryError> {
        let mut id: Option<(u16, u16, u16, u16)> = None;
        let mut name: Option<String> = None;
        let mut physical_path: Option<String> = None;
        let mut sys_path: Option<String> = None;
        let mut handlers: Option<Box<[String]>> = None;
        let mut bitmaps: HashMap<String, Box<[u64]>> = HashMap::new();

        let unparsed_properties = str
            .lines()
            .map(UnparsedQueryDeviceProperty::parse)
            .collect::<Result<Box<[_]>, _>>()?;
        let unparsed_properties = unparsed_properties
            .into_iter()
            .map(|prop| match prop.ident {
                'I' => {
                    let mut bus_type: Option<u16> = None;
                    let mut vendor: Option<u16> = None;
                    let mut product: Option<u16> = None;
                    let mut version: Option<u16> = None;
                    prop.content
                        .split_whitespace()
                        .flat_map(|s| s.split_once('='))
                        .for_each(|(n, v)| {
                            let Ok(v) = u16::from_str_radix(v, 16) else {
                                return;
                            };
                            match n.to_lowercase().as_str() {
                                "bus" => bus_type = Some(v),
                                "vendor" => vendor = Some(v),
                                "product" => product = Some(v),
                                "version" => version = Some(v),
                                _ => {}
                            }
                        });
                    if bus_type.is_none()
                        || vendor.is_none()
                        || product.is_none()
                        || version.is_none()
                    {
                        return Err(QueryError::Malformed("Malformed ID field"));
                    }
                    id = Some((
                        bus_type.unwrap(),
                        vendor.unwrap(),
                        product.unwrap(),
                        version.unwrap(),
                    ));
                    Ok(None)
                }
                'N' => {
                    let Some((_, [pname])) = Regex::new(r#"^[nN][aA][mM][eE]="(.*)"$"#)
                        .unwrap()
                        .captures(&prop.content)
                        .map(|i| i.extract())
                    else {
                        return Err(QueryError::Malformed("Malformed name field"));
                    };
                    name = Some(pname.to_owned());
                    Ok(None)
                }
                'P' => {
                    let Some((_, [ppath])) = Regex::new(r#"^[pP][hH][yY][sS]=(.*)$"#)
                        .unwrap()
                        .captures(&prop.content)
                        .map(|i| i.extract())
                    else {
                        return Err(QueryError::Malformed("Malformed physical path field"));
                    };
                    physical_path = Some(ppath.to_owned());
                    Ok(None)
                }
                'S' => {
                    let Some((_, [ppath])) = Regex::new(r#"^[sS][yY][sS][fF][sS]=(.*)$"#)
                        .unwrap()
                        .captures(&prop.content)
                        .map(|i| i.extract())
                    else {
                        return Err(QueryError::Malformed("Malformed sys path field"));
                    };
                    sys_path = Some(ppath.to_owned());
                    Ok(None)
                }
                'H' => {
                    let Some((_, phandlers)) = prop.content.split_once("=") else {
                        return Err(QueryError::Malformed("Malformed handlers field"));
                    };
                    handlers = Some(phandlers.split_whitespace().map(|s| s.to_owned()).collect());
                    Ok(None)
                }
                'B' => {
                    let Some((pname, pbits)) = prop.content.split_once("=") else {
                        return Err(QueryError::Malformed("Malformed bits field"));
                    };
                    if bitmaps.contains_key(pname) {
                        return Err(QueryError::Malformed("Duplicate bits field"));
                    }
                    bitmaps.insert(
                        pname.to_owned(),
                        pbits
                            .split_whitespace()
                            .map(|b| u64::from_str_radix(b, 16))
                            .collect::<Result<_, _>>()
                            .map_err(|_| QueryError::Malformed("Malformed bits field"))?,
                    );
                    Ok(None)
                }
                _ => Ok(Some(prop)),
            })
            .flat_map(|v| v.transpose())
            .collect::<Result<Box<[UnparsedQueryDeviceProperty]>, QueryError>>()?;

        let Some((id_bus_type, id_vendor, id_product, id_version)) = id else {
            return Err(QueryError::Malformed("Required ID field is missing"));
        };
        let Some(name) = name else {
            return Err(QueryError::Malformed("Required name field is missing"));
        };

        Ok(Self {
            id_bus_type,
            id_vendor,
            id_product,
            id_version,
            name,
            physical_path: physical_path.unwrap(),
            sys_path: sys_path.unwrap(),
            handlers: handlers.unwrap_or_default(),
            bitmaps,
            unparsed_properties,
        })
    }
}

pub fn query_devices() -> Result<Box<[QueryDevice]>, QueryError> {
    std::fs::read_to_string("/proc/bus/input/devices")?
        .split("\n\n")
        .filter(|s| !s.trim().is_empty())
        .map(QueryDevice::parse)
        .collect()
}

// #[cfg(test)]
// mod test {
//     use std::error::Error;
//
//     use crate::RAZER_DEVICE_VENDOR_ID;
//
//     use super::query_devices;
//
//     #[test]
//     fn query_test() -> Result<(), Box<dyn Error>> {
//         let mut query = query_devices()?.into_vec();
//         query.retain(|device| device.id_vendor == RAZER_DEVICE_VENDOR_ID);
//         println!("QUERY: {query:#?}");
//         Ok(())
//     }
// }
