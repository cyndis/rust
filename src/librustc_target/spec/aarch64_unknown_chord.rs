// Copyright 2018 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use spec::{LinkerFlavor, Target, TargetOptions, TargetResult, LinkArgs, RelroLevel,
    PanicStrategy};

pub fn target() -> TargetResult {
    let mut link_args = LinkArgs::new();
    link_args.insert(LinkerFlavor::Gcc, vec!["-nostartfiles".to_string()]);

    let objs = vec!["chord-crt0.o".to_string()];

    Ok(Target {
        arch: "aarch64".to_string(),
        data_layout: "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128".to_string(),
        llvm_target: "aarch64-none-elf".to_string(),
        target_endian: "little".to_string(),
        target_pointer_width: "64".to_string(),
        target_c_int_width: "32".to_string(),
        target_env: "none".to_string(),
        target_os: "chord".to_string(),
        target_vendor: "unknown".to_string(),
        linker_flavor: LinkerFlavor::Gcc,
        options: TargetOptions {
            linker: Some("aarch64-elf-gcc".to_string()),
            pre_link_args: link_args,
            dynamic_linking: true,
            executables: true,
            eliminate_frame_pointer: false,
            function_sections: true,
            exe_suffix: ".elf".to_string(),
            target_family: Some("chord".to_string()),
            linker_is_gnu: true,
            position_independent_executables: true,
            relro_level: RelroLevel::Full,
            pre_link_objects_exe: objs,
            max_atomic_width: Some(128),
            panic_strategy: PanicStrategy::Unwind,
            abi_blacklist: super::arm_base::abi_blacklist(),
            .. Default::default()
        }
    })
}
