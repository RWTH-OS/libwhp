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

use std;
use win_hv_platform::*;

// TODO (alexpilotti): transform into a macro
fn check_result(res: HRESULT) -> Result<(), HRESULT> {
    match res {
        S_OK => Ok(()),
        _ => Err(res),
    }
}

pub fn get_capability(capability_code: WHV_CAPABILITY_CODE) -> Result<WHV_CAPABILITY, HRESULT> {
    let mut capability: WHV_CAPABILITY;
    let mut written_size: UINT32 = 0;

    check_result(unsafe {
        capability = std::mem::zeroed();

        WHvGetCapability(
            capability_code,
            &mut capability as *mut _ as *mut VOID,
            std::mem::size_of::<WHV_CAPABILITY>() as UINT32,
            &mut written_size,
        )
    })?;
    Ok(capability)
}

pub struct Partition {
    partition: WHV_PARTITION_HANDLE,
}

impl Partition {
    pub fn new() -> Result<Partition, HRESULT> {
        let mut partition: WHV_PARTITION_HANDLE = std::ptr::null_mut();
        check_result(unsafe { WHvCreatePartition(&mut partition) })?;
        Ok(Partition {
            partition: partition,
        })
    }

    pub fn set_property(
        &mut self,
        property_code: WHV_PARTITION_PROPERTY_CODE,
        property: &WHV_PARTITION_PROPERTY,
    ) -> Result<(), HRESULT> {
        check_result(unsafe {
            WHvSetPartitionProperty(
                self.partition,
                property_code,
                property as *const _ as *const VOID,
                std::mem::size_of::<WHV_PARTITION_PROPERTY>() as UINT32,
            )
        })?;
        Ok(())
    }

    pub fn get_property(
        &mut self,
        property_code: WHV_PARTITION_PROPERTY_CODE,
    ) -> Result<WHV_PARTITION_PROPERTY, HRESULT> {
        let mut property: WHV_PARTITION_PROPERTY = unsafe { std::mem::zeroed() };
        let mut written_size: UINT32 = 0;

        check_result(unsafe {
            WHvGetPartitionProperty(
                self.partition,
                property_code,
                &mut property as *mut _ as *mut VOID,
                std::mem::size_of::<WHV_PARTITION_PROPERTY>() as UINT32,
                &mut written_size,
            )
        })?;
        Ok(property)
    }

    pub fn setup(&mut self) -> Result<(), HRESULT> {
        check_result(unsafe { WHvSetupPartition(self.partition) })?;
        Ok(())
    }

    pub fn create_virtual_processor(&mut self, index: UINT32) -> Result<VirtualProcessor, HRESULT> {
        check_result(unsafe { WHvCreateVirtualProcessor(self.partition, index, 0) })?;
        Ok(VirtualProcessor {
            partition: &self.partition,
            index: index,
        })
    }
}

impl Drop for Partition {
    fn drop(&mut self) {
        check_result(unsafe { WHvDeletePartition(self.partition) }).unwrap();
    }
}

pub struct VirtualProcessor<'a> {
    partition: &'a WHV_PARTITION_HANDLE,
    index: UINT32,
}

impl<'a> VirtualProcessor<'a> {
    pub fn run(&mut self) -> Result<WHV_RUN_VP_EXIT_CONTEXT, HRESULT> {
        let mut exit_context: WHV_RUN_VP_EXIT_CONTEXT = unsafe { std::mem::zeroed() };
        let exit_context_size = std::mem::size_of::<WHV_RUN_VP_EXIT_CONTEXT>() as UINT32;

        check_result(unsafe {
            WHvRunVirtualProcessor(
                *self.partition,
                self.index,
                &mut exit_context as *mut _ as *mut VOID,
                exit_context_size,
            )
        })?;
        Ok(exit_context)
    }

    pub fn cancel_run(&mut self) -> Result<(), HRESULT> {
        check_result(unsafe { WHvCancelRunVirtualProcessor(*self.partition, self.index, 0) })?;
        Ok(())
    }

    pub fn set_registers(
        &mut self,
        reg_names: &[WHV_REGISTER_NAME],
        reg_values: &[WHV_REGISTER_VALUE],
    ) -> Result<(), HRESULT> {
        let num_regs = reg_names.len();

        if num_regs != reg_values.len() {
            panic!("reg_names and reg_values must have the same length")
        }

        check_result(unsafe {
            WHvSetVirtualProcessorRegisters(
                *self.partition,
                self.index,
                reg_names.as_ptr(),
                num_regs as UINT32,
                reg_values.as_ptr(),
            )
        })?;

        Ok(())
    }

    pub fn get_registers(
        &mut self,
        reg_names: &[WHV_REGISTER_NAME],
        reg_values: &mut [WHV_REGISTER_VALUE],
    ) -> Result<(), HRESULT> {
        let num_regs = reg_names.len();

        if num_regs != reg_values.len() {
            panic!("reg_names and reg_values must have the same length")
        }

        check_result(unsafe {
            WHvGetVirtualProcessorRegisters(
                *self.partition,
                self.index,
                reg_names.as_ptr(),
                num_regs as UINT32,
                reg_values.as_mut_ptr(),
            )
        })?;
        Ok(())
    }
}

impl<'a> Drop for VirtualProcessor<'a> {
    fn drop(&mut self) {
        check_result(unsafe { WHvDeleteVirtualProcessor(*self.partition, self.index) }).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

    #[test]
    fn test_check_result_ok() {
        check_result(S_OK).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_check_result_fail() {
        check_result(E_INVALIDARG).unwrap();
    }

    #[test]
    fn test_create_delete_partition() {
        let p: Partition = Partition::new().unwrap();
        drop(p);
    }

    #[test]
    fn test_delete_partition_panic() {
        // Create an invalid partition
        let p: Partition = Partition {
            partition: std::ptr::null_mut(),
        };
        let result = std::panic::catch_unwind(|| drop(p));
        assert!(result.is_err(), "Drop was suppoesed to panic");
    }

    #[test]
    fn test_get_capability() {
        let _capability: WHV_CAPABILITY =
            get_capability(WHV_CAPABILITY_CODE::WHvCapabilityCodeHypervisorPresent).unwrap();
    }

    #[test]
    fn test_set_get_partition_property() {
        let mut p: Partition = Partition::new().unwrap();
        let property_code = WHV_PARTITION_PROPERTY_CODE::WHvPartitionPropertyCodeProcessorCount;
        let mut property: WHV_PARTITION_PROPERTY = unsafe { std::mem::zeroed() };
        property.ProcessorCount = 1;

        p.set_property(property_code, &property).unwrap();
        let property_out = p.get_property(property_code).unwrap();

        unsafe {
            assert_eq!(
                property.ProcessorCount, property_out.ProcessorCount,
                "The property value is not matching"
            );
        }
    }

    #[test]
    fn test_setup_partition() {
        let mut p: Partition = Partition::new().unwrap();
        let mut property: WHV_PARTITION_PROPERTY = unsafe { std::mem::zeroed() };
        property.ProcessorCount = 1;

        // Setup fails without setting at least the number of vcpus
        p.set_property(
            WHV_PARTITION_PROPERTY_CODE::WHvPartitionPropertyCodeProcessorCount,
            &property,
        ).unwrap();
        p.setup().unwrap();
    }

    #[test]
    fn test_setup_partition_fail() {
        let mut p: Partition = Partition::new().unwrap();
        match p.setup() {
            Err(e) => assert_eq!(e, WHV_E_INVALID_PARTITION_CONFIG, "Unexpected error code"),
            Ok(()) => panic!("An error was expected"),
        }
    }

    fn setup_vcpu_test(p: &mut Partition) {
        let mut property: WHV_PARTITION_PROPERTY = unsafe { std::mem::zeroed() };
        property.ProcessorCount = 1;

        p.set_property(
            WHV_PARTITION_PROPERTY_CODE::WHvPartitionPropertyCodeProcessorCount,
            &property,
        ).unwrap();
        p.setup().unwrap();
    }

    #[test]
    fn test_create_delete_virtual_processor() {
        let mut p: Partition = Partition::new().unwrap();
        setup_vcpu_test(&mut p);

        let vp_index: UINT32 = 0;
        let vp = p.create_virtual_processor(vp_index).unwrap();
        drop(vp)
    }

    #[test]
    fn test_run_virtual_processor() {
        let mut p: Partition = Partition::new().unwrap();
        setup_vcpu_test(&mut p);

        let vp_index: UINT32 = 0;
        let mut vp = p.create_virtual_processor(vp_index).unwrap();
        let mut _exit_context: WHV_RUN_VP_EXIT_CONTEXT = vp.run().unwrap();
    }

    #[test]
    fn test_cancel_virtual_processor() {
        let mut p: Partition = Partition::new().unwrap();
        setup_vcpu_test(&mut p);

        let vp_index: UINT32 = 0;
        let mut vp = p.create_virtual_processor(vp_index).unwrap();
        vp.cancel_run().unwrap();
    }

    #[test]
    fn test_set_get_virtual_processor_registers() {
        let mut p: Partition = Partition::new().unwrap();
        setup_vcpu_test(&mut p);

        let vp_index: UINT32 = 0;
        let mut vp = p.create_virtual_processor(vp_index).unwrap();

        const NUM_REGS: UINT32 = 1;
        const REG_VALUE: UINT64 = 11111111;
        let mut reg_names: [WHV_REGISTER_NAME; NUM_REGS as usize] = unsafe { std::mem::zeroed() };
        let mut reg_values: [WHV_REGISTER_VALUE; NUM_REGS as usize] = unsafe { std::mem::zeroed() };
        let mut reg_values_out: [WHV_REGISTER_VALUE; NUM_REGS as usize] =
            unsafe { std::mem::zeroed() };

        reg_names[0] = WHV_REGISTER_NAME::WHvX64RegisterRax;
        reg_values[0].Reg64 = REG_VALUE;

        vp.set_registers(&reg_names, &reg_values).unwrap();
        vp.get_registers(&reg_names, &mut reg_values_out).unwrap();

        unsafe {
            assert_eq!(
                reg_values_out[0].Reg64, REG_VALUE,
                "Registers values fo not match"
            );
        }
    }

}