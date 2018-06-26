// Copyright 2018 Cloudbase Solutions Srl
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License. You may obtain
// a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

#![allow(non_camel_case_types)]

use common_defs::*;
use win_hv_platform_defs::*;

pub type WHV_EMULATOR_HANDLE = *mut VOID;
pub type WHV_EMULATOR_STATUS = UINT32;

pub type WHV_EMULATOR_IO_PORT_CALLBACK =
    extern "stdcall" fn(Context: *const VOID, IoAccess: *mut WHV_EMULATOR_IO_ACCESS_INFO)
        -> HRESULT;

pub type WHV_EMULATOR_MEMORY_CALLBACK =
    extern "stdcall" fn(Context: *const VOID, MemoryAccess: *mut WHV_EMULATOR_MEMORY_ACCESS_INFO)
        -> HRESULT;

pub type WHV_EMULATOR_GET_VIRTUAL_PROCESSOR_REGISTERS_CALLBACK =
    extern "stdcall" fn(
        Context: *const VOID,
        RegisterNames: *const WHV_REGISTER_NAME,
        RegisterCount: UINT32,
        RegisterValues: *mut WHV_REGISTER_VALUE,
    ) -> HRESULT;

pub type WHV_EMULATOR_SET_VIRTUAL_PROCESSOR_REGISTERS_CALLBACK =
    extern "stdcall" fn(
        Context: *const VOID,
        RegisterNames: *const WHV_REGISTER_NAME,
        RegisterCount: UINT32,
        RegisterValues: *const WHV_REGISTER_VALUE,
    ) -> HRESULT;

pub type WHV_EMULATOR_TRANSLATE_GVA_PAGE_CALLBACK =
    extern "stdcall" fn(
        Context: *const VOID,
        Gva: WHV_GUEST_VIRTUAL_ADDRESS,
        TranslateFlags: WHV_TRANSLATE_GVA_FLAGS,
        TranslationResult: *mut WHV_TRANSLATE_GVA_RESULT_CODE,
        Gpa: *mut WHV_GUEST_PHYSICAL_ADDRESS,
    ) -> HRESULT;

#[derive(Copy, Clone)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct WHV_EMULATOR_MEMORY_ACCESS_INFO {
    pub GpaAddress: WHV_GUEST_PHYSICAL_ADDRESS,
    pub Direction: UINT8,
    pub AccessSize: UINT8,
    pub Data: [UINT8; 8],
}

#[derive(Copy, Clone)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct WHV_EMULATOR_IO_ACCESS_INFO {
    pub Direction: UINT8,
    pub Port: UINT16,
    pub AccessSize: UINT16,
    pub Data: UINT32,
}

#[derive(Copy, Clone)]
#[allow(non_snake_case)]
#[repr(C)]
pub struct WHV_EMULATOR_CALLBACKS {
    pub Size: UINT32,
    pub Reserved: UINT32,
    pub WHvEmulatorIoPortCallback: WHV_EMULATOR_IO_PORT_CALLBACK,
    pub WHvEmulatorMemoryCallback: WHV_EMULATOR_MEMORY_CALLBACK,
    pub WHvEmulatorGetVirtualProcessorRegisters:
        WHV_EMULATOR_GET_VIRTUAL_PROCESSOR_REGISTERS_CALLBACK,
    pub WHvEmulatorSetVirtualProcessorRegisters:
        WHV_EMULATOR_SET_VIRTUAL_PROCESSOR_REGISTERS_CALLBACK,
    pub WHvEmulatorTranslateGvaPage: WHV_EMULATOR_TRANSLATE_GVA_PAGE_CALLBACK,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

    #[test]
    fn test_data_type_sizes() {
        // Make sure all unions and structs have a size that matches the value
        // obtained with a sizeof() in C.
        assert_eq!(std::mem::size_of::<WHV_EMULATOR_MEMORY_ACCESS_INFO>(), 24);
        assert_eq!(std::mem::size_of::<WHV_EMULATOR_IO_ACCESS_INFO>(), 12);
        assert_eq!(std::mem::size_of::<WHV_EMULATOR_CALLBACKS>(), 48);
    }
}