enum ModCoderParts {
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

#[derive(Clone)]
pub struct ProcessorCoder {}

impl ProcessorCoder {
    pub fn new() -> Self {
        ProcessorCoder {}
    }
}