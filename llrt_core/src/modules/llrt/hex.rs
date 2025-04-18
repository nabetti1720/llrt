// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use rquickjs::{
    module::{Declarations, Exports, ModuleDef},
    prelude::Func,
    Ctx, Result, Value,
};

use crate::libs::{
    encoding::{bytes_from_hex, bytes_to_hex_string},
    utils::{
        bytes::{bytes_to_typed_array, ObjectBytes},
        module::{export_default, ModuleInfo},
        result::ResultExt,
    },
};

pub struct LlrtHexModule;

impl LlrtHexModule {
    pub fn encode<'js>(ctx: Ctx<'js>, bytes: ObjectBytes<'js>) -> Result<String> {
        Ok(bytes_to_hex_string(bytes.as_bytes(&ctx)?))
    }

    pub fn decode(ctx: Ctx, encoded: String) -> Result<Value> {
        let bytes = bytes_from_hex(encoded.as_bytes())
            .or_throw_msg(&ctx, "Cannot decode unrecognized sequence")?;

        bytes_to_typed_array(ctx, &bytes)
    }
}

impl ModuleDef for LlrtHexModule {
    fn declare(declare: &Declarations) -> Result<()> {
        declare.declare(stringify!(encode))?;
        declare.declare(stringify!(decode))?;
        declare.declare("default")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        export_default(ctx, exports, |default| {
            default.set(stringify!(encode), Func::from(Self::encode))?;
            default.set(stringify!(decode), Func::from(Self::decode))?;
            Ok(())
        })?;

        Ok(())
    }
}

impl From<LlrtHexModule> for ModuleInfo<LlrtHexModule> {
    fn from(val: LlrtHexModule) -> Self {
        ModuleInfo {
            name: "llrt:hex",
            module: val,
        }
    }
}
