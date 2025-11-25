use std::collections::HashMap;

#[repr(u8)]
pub enum ModCoderParts {
    HeadMod,
    UsedDefinedCode,
    HeadStruct,
    UserDefinedStruct,
    EndStruct,
    HeadBuilder,
    UserDefinedBuilder,
    EndBuilder,
    UserDefinedImplStruct,
    InitBody,
    RunBody,
    ProcessBody,
    StopBody,
}

impl TryFrom<u8> for ModCoderParts {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ModCoderParts::HeadMod),
            1 => Ok(ModCoderParts::UsedDefinedCode),
            2 => Ok(ModCoderParts::HeadStruct),
            3 => Ok(ModCoderParts::UserDefinedStruct),
            4 => Ok(ModCoderParts::EndStruct),
            5 => Ok(ModCoderParts::HeadBuilder),
            6 => Ok(ModCoderParts::UserDefinedBuilder),
            7 => Ok(ModCoderParts::EndBuilder),
            8 => Ok(ModCoderParts::UserDefinedImplStruct),
            9 => Ok(ModCoderParts::InitBody),
            10 => Ok(ModCoderParts::RunBody),
            11 => Ok(ModCoderParts::ProcessBody),
            12 => Ok(ModCoderParts::StopBody),
            _ => Err(()),
        }
    }
}
impl TryFrom<String> for ModCoderParts {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let num_result: Result<u8, _> = value.parse();
        let int_code: u8 = match num_result {
            Ok(n) => n,
            Err(_) => {return Err(());},
        };
        Self::try_from(int_code)
    }
}
#[derive(Clone)]
pub struct Limits {
    min: String,
    max: String,
}
#[derive(Clone)]
pub struct Typed {
    category: String,
    name: String,
    data_type: String,
    default: String,
    limits: Option<Limits>,
}
#[derive(Clone)]
pub struct ProcessorCoder {
    inputs: HashMap<String, String>,
    outputs: HashMap<String, String>,
    states: HashMap<String, String>,
    statics: HashMap<String, Vec<Typed>>,
    parameters: HashMap<String, Vec<Typed>>,
    path: String,
}

impl ProcessorCoder {
    pub fn new(path: String) -> Self {
        ProcessorCoder {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            states: HashMap::new(),
            statics: HashMap::new(),
            parameters: HashMap::new(),
            path,
        }
    }
    pub fn add_typed(&mut self, category: &String, name: &String, data_type: &String) {
        match category.as_str() {
            "input" => {
                self.inputs.insert(name.clone(), data_type.clone());
            },
            "output" => {
                self.outputs.insert(name.clone(), data_type.clone());
            },
            "state" => {
                self.states.insert(name.clone(), data_type.clone());
            },
            _ => {},
        }
    }
    pub fn add_settable(&mut self, category: &String, name: &String, data_type: &String, default: &String, limits: Option<&String>) {
        let settable = Typed {
            category: category.clone(),
            name: name.clone(),
            data_type: data_type.clone(),
            default: default.clone(),
            limits: limits.map(|lim_str| {
                let parts: Vec<&str> = lim_str.split(",").collect();
                Limits {
                    min: parts.get(0).unwrap_or(&"").to_string(),
                    max: parts.get(1).unwrap_or(&"").to_string(),
                }
            }),
        };
        match category.as_str() {
            "static" => {
                self.inputs.insert(name.clone(), data_type.clone());
            },
            "parameter" => {
                self.outputs.insert(name.clone(), data_type.clone());
            },
            _ => {},
        }
    }
    pub fn generate(&self) -> Result<(), String> {
        Ok(())
    }
}