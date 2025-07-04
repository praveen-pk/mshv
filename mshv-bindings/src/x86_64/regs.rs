// Copyright © 2020, Microsoft Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//

use crate::bindings::*;
use crate::HV_PAGE_SIZE;
#[cfg(feature = "with-serde")]
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::ptr;
use vmm_sys_util::errno;
use zerocopy::{FromBytes, IntoBytes};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct StandardRegisters {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct SegmentRegister {
    /* segment register + descriptor */
    pub base: u64,
    pub limit: u32,
    pub selector: u16,
    pub type_: u8,   /* type, writeable etc: 4 */
    pub present: u8, /* if not present, exception generated: 1 */
    pub dpl: u8,     /* descriptor privilege level (ring): 2 */
    pub db: u8,      /* default/big (16 or 32 bit size offset): 1 */
    pub s: u8,       /* non-system segment */
    pub l: u8,       /* long (64 bit): 1 */
    pub g: u8,       /* granularity (bytes or 4096 byte pages): 1 */
    pub avl: u8,     /* available (free bit for software to use): 1 */
    pub unusable: __u8,
    pub padding: __u8,
}

impl From<hv_x64_segment_register> for SegmentRegister {
    fn from(hv_reg: hv_x64_segment_register) -> Self {
        let mut reg = SegmentRegister {
            base: hv_reg.base,
            limit: hv_reg.limit,
            selector: hv_reg.selector,
            unusable: 0_u8,
            padding: 0_u8,
            ..Default::default()
        };

        // SAFETY: Getting a bunch of bitfields. Functions and unions are generated by bindgen
        // so we have to use unsafe here. We trust bindgen to generate the correct accessors.
        unsafe {
            reg.type_ = hv_reg.__bindgen_anon_1.__bindgen_anon_1.segment_type() as u8;
            reg.present = hv_reg.__bindgen_anon_1.__bindgen_anon_1.present() as u8;
            reg.dpl = hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .descriptor_privilege_level() as u8;
            reg.db = hv_reg.__bindgen_anon_1.__bindgen_anon_1._default() as u8;
            reg.s = hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .non_system_segment() as u8;
            reg.l = hv_reg.__bindgen_anon_1.__bindgen_anon_1._long() as u8;
            reg.g = hv_reg.__bindgen_anon_1.__bindgen_anon_1.granularity() as u8;
            reg.avl = hv_reg.__bindgen_anon_1.__bindgen_anon_1.available() as u8;
        }

        reg
    }
}
impl From<SegmentRegister> for hv_x64_segment_register {
    fn from(reg: SegmentRegister) -> Self {
        let mut hv_reg = hv_x64_segment_register {
            base: reg.base,
            limit: reg.limit,
            selector: reg.selector,
            ..Default::default()
        };

        // SAFETY: Setting a bunch of bitfields. Functions and unions are generated by bindgen
        // so we have to use unsafe here. We trust bindgen to generate the correct accessors.
        unsafe {
            hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .set_segment_type(reg.type_ as u16);
            hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .set_present(reg.present as u16);
            hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .set_descriptor_privilege_level(reg.dpl as u16);
            hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .set__default(reg.db as u16);
            hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .set_non_system_segment(reg.s as u16);
            hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .set__long(reg.l as u16);
            hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .set_granularity(reg.g as u16);
            hv_reg
                .__bindgen_anon_1
                .__bindgen_anon_1
                .set_available(reg.avl as u16);
        }

        hv_reg
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct TableRegister {
    pub base: u64,
    pub limit: u16,
}

impl From<hv_x64_table_register> for TableRegister {
    fn from(reg: hv_x64_table_register) -> Self {
        TableRegister {
            base: reg.base,
            limit: reg.limit,
        }
    }
}

impl From<TableRegister> for hv_x64_table_register {
    fn from(reg: TableRegister) -> Self {
        hv_x64_table_register {
            limit: reg.limit,
            base: reg.base,
            pad: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct SpecialRegisters {
    pub cs: SegmentRegister,
    pub ds: SegmentRegister,
    pub es: SegmentRegister,
    pub fs: SegmentRegister,
    pub gs: SegmentRegister,
    pub ss: SegmentRegister,
    pub tr: SegmentRegister,
    pub ldt: SegmentRegister,
    pub gdt: TableRegister,
    pub idt: TableRegister,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    pub efer: u64,
    pub apic_base: u64,
    pub interrupt_bitmap: [u64; 4usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct DebugRegisters {
    pub dr0: u64,
    pub dr1: u64,
    pub dr2: u64,
    pub dr3: u64,
    pub dr6: u64,
    pub dr7: u64,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct FloatingPointUnit {
    pub fpr: [[u8; 16usize]; 8usize],
    pub fcw: u16,
    pub fsw: u16,
    pub ftwx: u8,
    pub pad1: u8,
    pub last_opcode: u16,
    pub last_ip: u64,
    pub last_dp: u64,
    pub xmm: [[u8; 16usize]; 16usize],
    pub mxcsr: u32,
    pub pad2: u32,
}

pub const IA32_MSR_TSC: u32 = 0x00000010;
pub const IA32_MSR_EFER: u32 = 0xC0000080;
pub const IA32_MSR_KERNEL_GS_BASE: u32 = 0xC0000102;
pub const IA32_MSR_APIC_BASE: u32 = 0x0000001B;
pub const IA32_MSR_PAT: u32 = 0x0277;
pub const IA32_MSR_SYSENTER_CS: u32 = 0x00000174;
pub const IA32_MSR_SYSENTER_ESP: u32 = 0x00000175;
pub const IA32_MSR_SYSENTER_EIP: u32 = 0x00000176;
pub const IA32_MSR_STAR: u32 = 0xC0000081;
pub const IA32_MSR_LSTAR: u32 = 0xC0000082;
pub const IA32_MSR_CSTAR: u32 = 0xC0000083;
pub const IA32_MSR_SFMASK: u32 = 0xC0000084;

pub const IA32_MSR_MTRR_CAP: u32 = 0x00FE;
pub const IA32_MSR_MTRR_DEF_TYPE: u32 = 0x02FF;
pub const IA32_MSR_MTRR_PHYSBASE0: u32 = 0x0200;
pub const IA32_MSR_MTRR_PHYSMASK0: u32 = 0x0201;
pub const IA32_MSR_MTRR_PHYSBASE1: u32 = 0x0202;
pub const IA32_MSR_MTRR_PHYSMASK1: u32 = 0x0203;
pub const IA32_MSR_MTRR_PHYSBASE2: u32 = 0x0204;
pub const IA32_MSR_MTRR_PHYSMASK2: u32 = 0x0205;
pub const IA32_MSR_MTRR_PHYSBASE3: u32 = 0x0206;
pub const IA32_MSR_MTRR_PHYSMASK3: u32 = 0x0207;
pub const IA32_MSR_MTRR_PHYSBASE4: u32 = 0x0208;
pub const IA32_MSR_MTRR_PHYSMASK4: u32 = 0x0209;
pub const IA32_MSR_MTRR_PHYSBASE5: u32 = 0x020A;
pub const IA32_MSR_MTRR_PHYSMASK5: u32 = 0x020B;
pub const IA32_MSR_MTRR_PHYSBASE6: u32 = 0x020C;
pub const IA32_MSR_MTRR_PHYSMASK6: u32 = 0x020D;
pub const IA32_MSR_MTRR_PHYSBASE7: u32 = 0x020E;
pub const IA32_MSR_MTRR_PHYSMASK7: u32 = 0x020F;

pub const IA32_MSR_MTRR_FIX64K_00000: u32 = 0x0250;
pub const IA32_MSR_MTRR_FIX16K_80000: u32 = 0x0258;
pub const IA32_MSR_MTRR_FIX16K_A0000: u32 = 0x0259;
pub const IA32_MSR_MTRR_FIX4K_C0000: u32 = 0x0268;
pub const IA32_MSR_MTRR_FIX4K_C8000: u32 = 0x0269;
pub const IA32_MSR_MTRR_FIX4K_D0000: u32 = 0x026A;
pub const IA32_MSR_MTRR_FIX4K_D8000: u32 = 0x026B;
pub const IA32_MSR_MTRR_FIX4K_E0000: u32 = 0x026C;
pub const IA32_MSR_MTRR_FIX4K_E8000: u32 = 0x026D;
pub const IA32_MSR_MTRR_FIX4K_F0000: u32 = 0x026E;
pub const IA32_MSR_MTRR_FIX4K_F8000: u32 = 0x026F;

pub const IA32_MSR_TSC_AUX: u32 = 0xC0000103;
pub const IA32_MSR_BNDCFGS: u32 = 0x00000d90;
pub const IA32_MSR_DEBUG_CTL: u32 = 0x1D9;
pub const IA32_MSR_SPEC_CTRL: u32 = 0x00000048;
pub const IA32_MSR_TSC_ADJUST: u32 = 0x0000003b;

pub const IA32_MSR_MISC_ENABLE: u32 = 0x000001a0;
pub const MSR_IA32_SSP: u32 = 0x000007a0;
pub const MSR_IA32_U_CET: u32 = 0x000006a0; /* user mode cet */
pub const MSR_IA32_S_CET: u32 = 0x000006a2; /* kernel mode cet */
pub const MSR_IA32_PL0_SSP: u32 = 0x000006a4; /* ring-0 shadow stack pointer */
pub const MSR_IA32_PL1_SSP: u32 = 0x000006a5; /* ring-1 shadow stack pointer */
pub const MSR_IA32_PL2_SSP: u32 = 0x000006a6; /* ring-2 shadow stack pointer */
pub const MSR_IA32_PL3_SSP: u32 = 0x000006a7; /* ring-3 shadow stack pointer */
pub const MSR_IA32_INTERRUPT_SSP_TABLE_ADDR: u32 = 0x000006A8;
pub const MSR_IA32_REGISTER_U_XSS: u32 = 0x8008B;

pub fn msr_to_hv_reg_name(msr: u32) -> Result<::std::os::raw::c_uint, &'static str> {
    match msr {
        IA32_MSR_TSC => Ok(hv_register_name_HV_X64_REGISTER_TSC),

        IA32_MSR_EFER => Ok(hv_register_name_HV_X64_REGISTER_EFER),
        IA32_MSR_KERNEL_GS_BASE => Ok(hv_register_name_HV_X64_REGISTER_KERNEL_GS_BASE),
        IA32_MSR_APIC_BASE => Ok(hv_register_name_HV_X64_REGISTER_APIC_BASE),
        IA32_MSR_PAT => Ok(hv_register_name_HV_X64_REGISTER_PAT),
        IA32_MSR_SYSENTER_CS => Ok(hv_register_name_HV_X64_REGISTER_SYSENTER_CS),
        IA32_MSR_SYSENTER_ESP => Ok(hv_register_name_HV_X64_REGISTER_SYSENTER_ESP),
        IA32_MSR_SYSENTER_EIP => Ok(hv_register_name_HV_X64_REGISTER_SYSENTER_EIP),
        IA32_MSR_STAR => Ok(hv_register_name_HV_X64_REGISTER_STAR),
        IA32_MSR_LSTAR => Ok(hv_register_name_HV_X64_REGISTER_LSTAR),
        IA32_MSR_CSTAR => Ok(hv_register_name_HV_X64_REGISTER_CSTAR),
        IA32_MSR_SFMASK => Ok(hv_register_name_HV_X64_REGISTER_SFMASK),

        IA32_MSR_MTRR_CAP => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_CAP),
        IA32_MSR_MTRR_DEF_TYPE => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_DEF_TYPE),
        IA32_MSR_MTRR_PHYSBASE0 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_BASE0),
        IA32_MSR_MTRR_PHYSMASK0 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_MASK0),
        IA32_MSR_MTRR_PHYSBASE1 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_BASE1),
        IA32_MSR_MTRR_PHYSMASK1 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_MASK1),
        IA32_MSR_MTRR_PHYSBASE2 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_BASE2),
        IA32_MSR_MTRR_PHYSMASK2 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_MASK2),
        IA32_MSR_MTRR_PHYSBASE3 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_BASE3),
        IA32_MSR_MTRR_PHYSMASK3 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_MASK3),
        IA32_MSR_MTRR_PHYSBASE4 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_BASE4),
        IA32_MSR_MTRR_PHYSMASK4 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_MASK4),
        IA32_MSR_MTRR_PHYSBASE5 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_BASE5),
        IA32_MSR_MTRR_PHYSMASK5 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_MASK5),
        IA32_MSR_MTRR_PHYSBASE6 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_BASE6),
        IA32_MSR_MTRR_PHYSMASK6 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_MASK6),
        IA32_MSR_MTRR_PHYSBASE7 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_BASE7),
        IA32_MSR_MTRR_PHYSMASK7 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_PHYS_MASK7),

        IA32_MSR_MTRR_FIX64K_00000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX64K00000),
        IA32_MSR_MTRR_FIX16K_80000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX16K80000),
        IA32_MSR_MTRR_FIX16K_A0000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX16KA0000),
        IA32_MSR_MTRR_FIX4K_C0000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX4KC0000),
        IA32_MSR_MTRR_FIX4K_C8000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX4KC8000),
        IA32_MSR_MTRR_FIX4K_D0000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX4KD0000),
        IA32_MSR_MTRR_FIX4K_D8000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX4KD8000),
        IA32_MSR_MTRR_FIX4K_E0000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX4KE0000),
        IA32_MSR_MTRR_FIX4K_E8000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX4KE8000),
        IA32_MSR_MTRR_FIX4K_F0000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX4KF0000),
        IA32_MSR_MTRR_FIX4K_F8000 => Ok(hv_register_name_HV_X64_REGISTER_MSR_MTRR_FIX4KF8000),

        IA32_MSR_TSC_AUX => Ok(hv_register_name_HV_X64_REGISTER_TSC_AUX),
        IA32_MSR_BNDCFGS => Ok(hv_register_name_HV_X64_REGISTER_BNDCFGS),
        IA32_MSR_DEBUG_CTL => Ok(hv_register_name_HV_X64_REGISTER_DEBUG_CTL),
        IA32_MSR_TSC_ADJUST => Ok(hv_register_name_HV_X64_REGISTER_TSC_ADJUST),
        IA32_MSR_SPEC_CTRL => Ok(hv_register_name_HV_X64_REGISTER_SPEC_CTRL),
        HV_X64_MSR_GUEST_OS_ID => Ok(hv_register_name_HV_REGISTER_GUEST_OS_ID),
        HV_X64_MSR_SINT0 => Ok(hv_register_name_HV_REGISTER_SINT0),
        HV_X64_MSR_SINT1 => Ok(hv_register_name_HV_REGISTER_SINT1),
        HV_X64_MSR_SINT2 => Ok(hv_register_name_HV_REGISTER_SINT2),
        HV_X64_MSR_SINT3 => Ok(hv_register_name_HV_REGISTER_SINT3),
        HV_X64_MSR_SINT4 => Ok(hv_register_name_HV_REGISTER_SINT4),
        HV_X64_MSR_SINT5 => Ok(hv_register_name_HV_REGISTER_SINT5),
        HV_X64_MSR_SINT6 => Ok(hv_register_name_HV_REGISTER_SINT6),
        HV_X64_MSR_SINT7 => Ok(hv_register_name_HV_REGISTER_SINT7),
        HV_X64_MSR_SINT8 => Ok(hv_register_name_HV_REGISTER_SINT8),
        HV_X64_MSR_SINT9 => Ok(hv_register_name_HV_REGISTER_SINT9),
        HV_X64_MSR_SINT10 => Ok(hv_register_name_HV_REGISTER_SINT10),
        HV_X64_MSR_SINT11 => Ok(hv_register_name_HV_REGISTER_SINT11),
        HV_X64_MSR_SINT12 => Ok(hv_register_name_HV_REGISTER_SINT12),
        HV_X64_MSR_SINT13 => Ok(hv_register_name_HV_REGISTER_SINT13),
        HV_X64_MSR_SINT14 => Ok(hv_register_name_HV_REGISTER_SINT14),
        HV_X64_MSR_SINT15 => Ok(hv_register_name_HV_REGISTER_SINT15),
        IA32_MSR_MISC_ENABLE => Ok(hv_register_name_HV_X64_REGISTER_MSR_IA32_MISC_ENABLE),
        HV_X64_MSR_SCONTROL => Ok(hv_register_name_HV_REGISTER_SCONTROL),
        HV_X64_MSR_SIEFP => Ok(hv_register_name_HV_REGISTER_SIEFP),
        HV_X64_MSR_SIMP => Ok(hv_register_name_HV_REGISTER_SIMP),
        HV_X64_MSR_REFERENCE_TSC => Ok(hv_register_name_HV_REGISTER_REFERENCE_TSC),
        HV_X64_MSR_EOM => Ok(hv_register_name_HV_REGISTER_EOM),
        MSR_IA32_REGISTER_U_XSS => Ok(hv_register_name_HV_X64_REGISTER_U_XSS),
        MSR_IA32_U_CET => Ok(hv_register_name_HV_X64_REGISTER_U_CET),
        MSR_IA32_S_CET => Ok(hv_register_name_HV_X64_REGISTER_S_CET),
        MSR_IA32_SSP => Ok(hv_register_name_HV_X64_REGISTER_SSP),
        MSR_IA32_PL0_SSP => Ok(hv_register_name_HV_X64_REGISTER_PL0_SSP),
        MSR_IA32_PL1_SSP => Ok(hv_register_name_HV_X64_REGISTER_PL1_SSP),
        MSR_IA32_PL2_SSP => Ok(hv_register_name_HV_X64_REGISTER_PL2_SSP),
        MSR_IA32_PL3_SSP => Ok(hv_register_name_HV_X64_REGISTER_PL3_SSP),
        MSR_IA32_INTERRUPT_SSP_TABLE_ADDR => {
            Ok(hv_register_name_HV_X64_REGISTER_INTERRUPT_SSP_TABLE_ADDR)
        }
        _ => Err("Not a supported hv_register_name msr"),
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct msr_entry {
    pub index: u32,
    pub reserved: u32,
    pub data: u64,
}

#[repr(C)]
#[derive(Debug, Default)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct msrs {
    pub nmsrs: u32,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub pad: u32,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub entries: __IncompleteArrayField<msr_entry>,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct msr_list {
    pub nmsrs: u32,
    pub indices: __IncompleteArrayField<u32>,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct VcpuEvents {
    pub pending_interruption: u64,
    pub interrupt_state: u64,
    pub internal_activity_state: u64,
    pub pending_event0: [u8; 16usize],
    pub pending_event1: [u8; 16usize],
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct Xcrs {
    pub xcr0: u64,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
pub struct hv_cpuid_entry {
    pub function: __u32,
    pub index: __u32,
    pub flags: __u32,
    pub eax: __u32,
    pub ebx: __u32,
    pub ecx: __u32,
    pub edx: __u32,
    pub padding: [__u32; 3usize],
}

#[repr(C)]
#[derive(Debug, Default)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct hv_cpuid {
    pub nent: __u32,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub padding: __u32,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub entries: __IncompleteArrayField<hv_cpuid_entry>,
}

pub const LOCAL_APIC_OFFSET_APIC_ID: isize = 0x20; // APIC ID Register.
pub const LOCAL_APIC_OFFSET_VERSION: isize = 0x30; // APIC Version Register.
pub const LOCAL_APIC_OFFSET_TPR: isize = 0x80; // Task Priority Register
pub const LOCAL_APIC_OFFSET_APR: isize = 0x90; // Arbitration Priority Register.
pub const LOCAL_APIC_OFFSET_PPR: isize = 0xA0; // Processor Priority Register.
pub const LOCAL_APIC_OFFSET_EOI: isize = 0xB0; // End Of Interrupt Register.
pub const LOCAL_APIC_OFFSET_REMOTE_READ: isize = 0xC0; // Remote Read Register
pub const LOCAL_APIC_OFFSET_LDR: isize = 0xD0; // Logical Destination Register.
pub const LOCAL_APIC_OFFSET_DFR: isize = 0xE0; // Destination Format Register.
pub const LOCAL_APIC_OFFSET_SPURIOUS: isize = 0xF0; // Spurious Interrupt Vector.
pub const LOCAL_APIC_OFFSET_ISR: isize = 0x100; // In-Service Register.
pub const LOCAL_APIC_OFFSET_TMR: isize = 0x180; // Trigger Mode Register.
pub const LOCAL_APIC_OFFSET_IRR: isize = 0x200; // Interrupt Request Register.
pub const LOCAL_APIC_OFFSET_ERROR: isize = 0x280; // Error Status Register.
pub const LOCAL_APIC_OFFSET_ICR_LOW: isize = 0x300; // ICR Low.
pub const LOCAL_APIC_OFFSET_ICR_HIGH: isize = 0x310; // ICR High.
pub const LOCAL_APIC_OFFSET_TIMER_LVT: isize = 0x320; // LVT Timer Register.
pub const LOCAL_APIC_OFFSET_THERMAL_LVT: isize = 0x330; // LVT Thermal Register.
pub const LOCAL_APIC_OFFSET_PERFMON_LVT: isize = 0x340; // LVT Performance Monitor Register.
pub const LOCAL_APIC_OFFSET_LINT0_LVT: isize = 0x350; // LVT Local Int0; Register.
pub const LOCAL_APIC_OFFSET_LINT1_LVT: isize = 0x360; // LVT Local Int1 Register.
pub const LOCAL_APIC_OFFSET_ERROR_LVT: isize = 0x370; // LVT Error Register.
pub const LOCAL_APIC_OFFSET_INITIAL_COUNT: isize = 0x380; // Initial count Register.
pub const LOCAL_APIC_OFFSET_CURRENT_COUNT: isize = 0x390; // R/O Current count Register.
pub const LOCAL_APIC_OFFSET_DIVIDER: isize = 0x3e0; // Divide configuration Register.
pub const LOCAL_X2APIC_OFFSET_SELF_IPI: isize = 0x3f0; // Self IPI register, only present in x2APIC.

pub struct Buffer {
    pub layout: std::alloc::Layout,
    pub buf: *mut u8,
}

impl Buffer {
    pub fn new(size: usize, align: usize) -> Result<Buffer, errno::Error> {
        let layout = std::alloc::Layout::from_size_align(size, align).unwrap();
        // SAFETY: layout is valid
        let buf = unsafe { std::alloc::alloc(layout) };
        if buf.is_null() {
            return Err(errno::Error::new(libc::ENOMEM));
        }

        let buf = Buffer { layout, buf };

        Ok(buf)
    }

    pub fn dealloc(&mut self) {
        // SAFETY: buf was allocated with layout
        unsafe {
            std::alloc::dealloc(self.buf, self.layout);
        }
    }

    pub fn size(&self) -> usize {
        self.layout.size()
    }

    pub fn zero_out_buf(&mut self) {
        // SAFETY: We write zeros to a valid pointer and the size is valid and allocated from a valid layout.
        unsafe {
            ::std::ptr::write_bytes(self.buf, 0u8, self.size());
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.dealloc();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, FromBytes)]
/// Fixed buffer for lapic state
pub struct LapicState {
    pub regs: [::std::os::raw::c_char; 1024usize],
}

impl Default for LapicState {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, FromBytes)]
/// Fixed buffer for xsave state
pub struct XSave {
    pub buffer: [u8; 4096usize],
}

impl Default for XSave {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl TryFrom<Buffer> for XSave {
    type Error = errno::Error;
    fn try_from(buf: Buffer) -> Result<Self, Self::Error> {
        let mut ret = XSave {
            ..Default::default()
        };
        let ret_size = std::mem::size_of_val(&ret.buffer);
        if ret_size < buf.size() {
            return Err(errno::Error::new(libc::EINVAL));
        }
        // SAFETY: ret is large enough to hold buffer
        unsafe { ptr::copy(buf.buf, ret.buffer.as_mut_ptr(), buf.size()) };
        Ok(ret)
    }
}

impl TryFrom<&XSave> for Buffer {
    type Error = errno::Error;
    fn try_from(reg: &XSave) -> Result<Self, Self::Error> {
        let reg_size = std::mem::size_of_val(&reg.buffer);
        let num_pages = (reg_size + HV_PAGE_SIZE - 1) >> HV_HYP_PAGE_SHIFT;
        let buffer = Buffer::new(num_pages * HV_PAGE_SIZE, HV_PAGE_SIZE)?;
        // SAFETY: buffer is large enough to hold reg
        unsafe { ptr::copy(reg.buffer.as_ptr(), buffer.buf, reg_size) };
        Ok(buffer)
    }
}

impl TryFrom<Buffer> for LapicState {
    type Error = errno::Error;
    fn try_from(buf: Buffer) -> Result<Self, Self::Error> {
        let mut ret: LapicState = LapicState::default();
        let state = ret.regs.as_mut_ptr();
        if buf.size() < std::mem::size_of::<hv_local_interrupt_controller_state>() {
            return Err(errno::Error::new(libc::EINVAL));
        }
        // SAFETY: buf is large enough for hv_local_interrupt_controller_state
        unsafe {
            let hv_state = &*(buf.buf as *const hv_local_interrupt_controller_state);
            *(state.offset(LOCAL_APIC_OFFSET_APIC_ID) as *mut u32) = hv_state.apic_id;
            *(state.offset(LOCAL_APIC_OFFSET_VERSION) as *mut u32) = hv_state.apic_version;
            *(state.offset(LOCAL_APIC_OFFSET_REMOTE_READ) as *mut u32) = hv_state.apic_remote_read;
            *(state.offset(LOCAL_APIC_OFFSET_LDR) as *mut u32) = hv_state.apic_ldr;
            *(state.offset(LOCAL_APIC_OFFSET_DFR) as *mut u32) = hv_state.apic_dfr;
            *(state.offset(LOCAL_APIC_OFFSET_SPURIOUS) as *mut u32) = hv_state.apic_spurious;
            *(state.offset(LOCAL_APIC_OFFSET_ERROR) as *mut u32) = hv_state.apic_esr;
            *(state.offset(LOCAL_APIC_OFFSET_ICR_LOW) as *mut u32) = hv_state.apic_icr_low;
            *(state.offset(LOCAL_APIC_OFFSET_ICR_HIGH) as *mut u32) = hv_state.apic_icr_high;
            *(state.offset(LOCAL_APIC_OFFSET_TIMER_LVT) as *mut u32) = hv_state.apic_lvt_timer;
            *(state.offset(LOCAL_APIC_OFFSET_THERMAL_LVT) as *mut u32) = hv_state.apic_lvt_thermal;
            *(state.offset(LOCAL_APIC_OFFSET_PERFMON_LVT) as *mut u32) = hv_state.apic_lvt_perfmon;
            *(state.offset(LOCAL_APIC_OFFSET_LINT0_LVT) as *mut u32) = hv_state.apic_lvt_lint0;
            *(state.offset(LOCAL_APIC_OFFSET_LINT1_LVT) as *mut u32) = hv_state.apic_lvt_lint1;
            *(state.offset(LOCAL_APIC_OFFSET_ERROR_LVT) as *mut u32) = hv_state.apic_lvt_error;
            *(state.offset(LOCAL_APIC_OFFSET_INITIAL_COUNT) as *mut u32) =
                hv_state.apic_initial_count;
            *(state.offset(LOCAL_APIC_OFFSET_CURRENT_COUNT) as *mut u32) =
                hv_state.apic_counter_value;
            *(state.offset(LOCAL_APIC_OFFSET_DIVIDER) as *mut u32) =
                hv_state.apic_divide_configuration;

            /* vectors ISR TMR IRR */
            for i in 0..8 {
                *(state.offset(LOCAL_APIC_OFFSET_ISR + i * 16) as *mut u32) =
                    hv_state.apic_isr[i as usize];
                *(state.offset(LOCAL_APIC_OFFSET_TMR + i * 16) as *mut u32) =
                    hv_state.apic_tmr[i as usize];
                *(state.offset(LOCAL_APIC_OFFSET_IRR + i * 16) as *mut u32) =
                    hv_state.apic_irr[i as usize];
            }

            // Highest priority interrupt (isr = in service register) this is how WHP computes it
            let mut isrv: u32 = 0;
            for i in (0..8).rev() {
                let val: u32 = hv_state.apic_isr[i as usize];
                if val != 0 {
                    isrv = 31 - val.leading_zeros(); // index of most significant set bit
                    isrv += i * 4 * 8; // i don't know
                    break;
                }
            }

            // TODO This is meant to be max(tpr, isrv), but tpr is not populated!
            *(state.offset(LOCAL_APIC_OFFSET_PPR) as *mut u32) = isrv;
        }
        Ok(ret)
    }
}

impl TryFrom<&LapicState> for Buffer {
    type Error = errno::Error;
    fn try_from(reg: &LapicState) -> Result<Self, Self::Error> {
        let hv_state_size = std::mem::size_of::<hv_local_interrupt_controller_state>();
        let num_pages = (hv_state_size + HV_PAGE_SIZE - 1) >> HV_HYP_PAGE_SHIFT;
        let buffer = Buffer::new(num_pages * HV_PAGE_SIZE, HV_PAGE_SIZE)?;
        // SAFETY: buf is large enough for hv_local_interrupt_controller_state
        unsafe {
            let state = reg.regs.as_ptr();
            let hv_state = &mut *(buffer.buf as *mut hv_local_interrupt_controller_state);
            *hv_state = hv_local_interrupt_controller_state {
                apic_id: *(state.offset(LOCAL_APIC_OFFSET_APIC_ID) as *mut u32),
                apic_version: *(state.offset(LOCAL_APIC_OFFSET_VERSION) as *mut u32),
                apic_remote_read: *(state.offset(LOCAL_APIC_OFFSET_REMOTE_READ) as *mut u32),
                apic_ldr: *(state.offset(LOCAL_APIC_OFFSET_LDR) as *mut u32),
                apic_dfr: *(state.offset(LOCAL_APIC_OFFSET_DFR) as *mut u32),
                apic_spurious: *(state.offset(LOCAL_APIC_OFFSET_SPURIOUS) as *mut u32),
                apic_esr: *(state.offset(LOCAL_APIC_OFFSET_ERROR) as *mut u32),
                apic_icr_low: *(state.offset(LOCAL_APIC_OFFSET_ICR_LOW) as *mut u32),
                apic_icr_high: *(state.offset(LOCAL_APIC_OFFSET_ICR_HIGH) as *mut u32),
                apic_lvt_timer: *(state.offset(LOCAL_APIC_OFFSET_TIMER_LVT) as *mut u32),
                apic_lvt_thermal: *(state.offset(LOCAL_APIC_OFFSET_THERMAL_LVT) as *mut u32),
                apic_lvt_perfmon: *(state.offset(LOCAL_APIC_OFFSET_PERFMON_LVT) as *mut u32),
                apic_lvt_lint0: *(state.offset(LOCAL_APIC_OFFSET_LINT0_LVT) as *mut u32),
                apic_lvt_lint1: *(state.offset(LOCAL_APIC_OFFSET_LINT1_LVT) as *mut u32),
                apic_lvt_error: *(state.offset(LOCAL_APIC_OFFSET_ERROR_LVT) as *mut u32),
                apic_initial_count: *(state.offset(LOCAL_APIC_OFFSET_INITIAL_COUNT) as *mut u32),
                apic_counter_value: *(state.offset(LOCAL_APIC_OFFSET_CURRENT_COUNT) as *mut u32),
                apic_divide_configuration: *(state.offset(LOCAL_APIC_OFFSET_DIVIDER) as *mut u32),
                apic_error_status: 0,
                apic_lvt_cmci: 0,
                apic_isr: [0; 8],
                apic_tmr: [0; 8],
                apic_irr: [0; 8],
            };

            /* vectors ISR TMR IRR */
            for i in 0..8 {
                hv_state.apic_isr[i as usize] =
                    *(state.offset(LOCAL_APIC_OFFSET_ISR + i * 16) as *mut u32);
                hv_state.apic_tmr[i as usize] =
                    *(state.offset(LOCAL_APIC_OFFSET_TMR + i * 16) as *mut u32);
                hv_state.apic_irr[i as usize] =
                    *(state.offset(LOCAL_APIC_OFFSET_IRR + i * 16) as *mut u32);
            }
        }

        Ok(buffer)
    }
}

// implement `Display` for `XSave`
impl fmt::Display for XSave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "buffer: {:?}\n data: {:02X?}",
            self.buffer.as_ptr(),
            self.buffer,
        )
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct SuspendRegisters {
    pub explicit_register: u64,
    pub intercept_register: u64,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, IntoBytes, FromBytes)]
#[cfg_attr(feature = "with-serde", derive(Deserialize, Serialize))]
pub struct MiscRegs {
    pub hypercall: u64,
    pub int_vec: u64,
}

const fn initialize_comp_sizes() -> [usize; MSHV_VP_STATE_COUNT as usize] {
    let mut vp_state_comp_size = [0; MSHV_VP_STATE_COUNT as usize];

    vp_state_comp_size[MSHV_VP_STATE_LAPIC as usize] = std::mem::size_of::<LapicState>();
    vp_state_comp_size[MSHV_VP_STATE_XSAVE as usize] = std::mem::size_of::<XSave>();
    vp_state_comp_size[MSHV_VP_STATE_SIMP as usize] = HV_PAGE_SIZE; // Assuming SIMP page is
                                                                    // allocated by the Hypervisor
                                                                    // which is of PAGE_SIZE
    vp_state_comp_size[MSHV_VP_STATE_SIEFP as usize] = HV_PAGE_SIZE; // Assuming SIEFP page is
                                                                     // allocated by the Hypervisor
                                                                     // which is of PAGE_SIZE
    vp_state_comp_size[MSHV_VP_STATE_SYNTHETIC_TIMERS as usize] =
        std::mem::size_of::<hv_synthetic_timers_state>();

    vp_state_comp_size
}

// Total size: 13512 bytes
// 1. MSHV_VP_STATE_LAPIC, Size: 1024 bytes;
// 2. MSHV_VP_STATE_XSAVE, Size: 4096 bytes;
// 3. MSHV_VP_STATE_SIMP, Size: 4096 bytes;
// 4. MSHV_VP_STATE_SIEFP, Size: 4096 bytes;
// 5. MSHV_VP_STATE_SYNTHETIC_TIMERS, Size: 200 bytes;
const VP_STATE_COMP_SIZES: [usize; MSHV_VP_STATE_COUNT as usize] = initialize_comp_sizes();

pub const VP_STATE_COMPONENTS_BUFFER_SIZE: usize = VP_STATE_COMP_SIZES
    [MSHV_VP_STATE_LAPIC as usize]
    + VP_STATE_COMP_SIZES[MSHV_VP_STATE_XSAVE as usize]
    + VP_STATE_COMP_SIZES[MSHV_VP_STATE_SIMP as usize]
    + VP_STATE_COMP_SIZES[MSHV_VP_STATE_SIEFP as usize]
    + VP_STATE_COMP_SIZES[MSHV_VP_STATE_SYNTHETIC_TIMERS as usize];

#[inline(always)]
fn get_vp_state_comp_start_offset(index: usize) -> usize {
    VP_STATE_COMP_SIZES[0..index].iter().copied().sum()
}

// Total five components are stored in a single buffer serially
// Components are:
// Local APIC, Xsave, Synthetic Message Page, Synthetic Event Flags Page
// and Synthetic Timers.
#[repr(C)]
#[derive(Copy, Clone, Debug, IntoBytes, FromBytes)]
/// Fixed buffer for VP state components
pub struct AllVpStateComponents {
    pub buffer: [u8; VP_STATE_COMPONENTS_BUFFER_SIZE],
}

impl Default for AllVpStateComponents {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl AllVpStateComponents {
    pub fn copy_to_or_from_buffer(&mut self, index: usize, buffer: &mut Buffer, to_buffer: bool) {
        let len: usize = VP_STATE_COMP_SIZES[index];

        if len > buffer.size() {
            panic!("Invalid buffer length for state components");
        }

        let start = get_vp_state_comp_start_offset(index);
        let end = start + len;

        if to_buffer {
            // SAFETY: buffer is large enough to hold state data
            unsafe { ptr::copy(self.buffer[start..end].as_ptr(), buffer.buf, len) };
        } else {
            // SAFETY: buffer is large enough to hold state data
            unsafe { ptr::copy(buffer.buf, self.buffer[start..end].as_mut_ptr(), len) };
        }
    }
}

#[macro_export]
macro_rules! set_gp_regs_field_ptr {
    ($this: ident, $name: ident, $value: expr) => {
        #[allow(clippy::macro_metavars_in_unsafe)]
        // SAFETY: access union fields
        unsafe {
            (*$this)
                .__bindgen_anon_1
                .__bindgen_anon_1
                .__bindgen_anon_1
                .__bindgen_anon_1
                .$name = $value;
        }
    };
}

#[macro_export]
macro_rules! get_gp_regs_field_ptr {
    ($this: ident, $name: ident) => {
        (*$this)
            .__bindgen_anon_1
            .__bindgen_anon_1
            .__bindgen_anon_1
            .__bindgen_anon_1
            .$name
    };
}

pub static MSRS_SYNIC: &[u32; 19] = &[
    HV_X64_MSR_SINT0,
    HV_X64_MSR_SINT1,
    HV_X64_MSR_SINT2,
    HV_X64_MSR_SINT3,
    HV_X64_MSR_SINT4,
    HV_X64_MSR_SINT5,
    HV_X64_MSR_SINT6,
    HV_X64_MSR_SINT7,
    HV_X64_MSR_SINT8,
    HV_X64_MSR_SINT9,
    HV_X64_MSR_SINT10,
    HV_X64_MSR_SINT11,
    HV_X64_MSR_SINT12,
    HV_X64_MSR_SINT13,
    HV_X64_MSR_SINT14,
    HV_X64_MSR_SINT15,
    HV_X64_MSR_SCONTROL,
    HV_X64_MSR_SIEFP,
    HV_X64_MSR_SIMP,
];

pub static MSRS_COMMON: &[u32; 42] = &[
    IA32_MSR_TSC,
    IA32_MSR_EFER,
    IA32_MSR_KERNEL_GS_BASE,
    IA32_MSR_APIC_BASE,
    IA32_MSR_PAT,
    IA32_MSR_SYSENTER_CS,
    IA32_MSR_SYSENTER_ESP,
    IA32_MSR_SYSENTER_EIP,
    IA32_MSR_STAR,
    IA32_MSR_LSTAR,
    IA32_MSR_CSTAR,
    IA32_MSR_SFMASK,
    IA32_MSR_MTRR_DEF_TYPE,
    IA32_MSR_MTRR_PHYSBASE0,
    IA32_MSR_MTRR_PHYSMASK0,
    IA32_MSR_MTRR_PHYSBASE1,
    IA32_MSR_MTRR_PHYSMASK1,
    IA32_MSR_MTRR_PHYSBASE2,
    IA32_MSR_MTRR_PHYSMASK2,
    IA32_MSR_MTRR_PHYSBASE3,
    IA32_MSR_MTRR_PHYSMASK3,
    IA32_MSR_MTRR_PHYSBASE4,
    IA32_MSR_MTRR_PHYSMASK4,
    IA32_MSR_MTRR_PHYSBASE5,
    IA32_MSR_MTRR_PHYSMASK5,
    IA32_MSR_MTRR_PHYSBASE6,
    IA32_MSR_MTRR_PHYSMASK6,
    IA32_MSR_MTRR_PHYSBASE7,
    IA32_MSR_MTRR_PHYSMASK7,
    IA32_MSR_MTRR_FIX64K_00000,
    IA32_MSR_MTRR_FIX16K_80000,
    IA32_MSR_MTRR_FIX16K_A0000,
    IA32_MSR_MTRR_FIX4K_C0000,
    IA32_MSR_MTRR_FIX4K_C8000,
    IA32_MSR_MTRR_FIX4K_D0000,
    IA32_MSR_MTRR_FIX4K_D8000,
    IA32_MSR_MTRR_FIX4K_E0000,
    IA32_MSR_MTRR_FIX4K_E8000,
    IA32_MSR_MTRR_FIX4K_F0000,
    IA32_MSR_MTRR_FIX4K_F8000,
    IA32_MSR_DEBUG_CTL,
    HV_X64_MSR_EOM,
];

pub static MSRS_CET_SS: &[u32; 8] = &[
    MSR_IA32_U_CET,
    MSR_IA32_S_CET,
    MSR_IA32_SSP,
    MSR_IA32_PL0_SSP,
    MSR_IA32_PL1_SSP,
    MSR_IA32_PL2_SSP,
    MSR_IA32_PL3_SSP,
    MSR_IA32_INTERRUPT_SSP_TABLE_ADDR,
];

pub static MSRS_OTHER: &[u32; 4] = &[
    MSR_IA32_REGISTER_U_XSS,
    IA32_MSR_TSC_AUX,
    HV_X64_MSR_REFERENCE_TSC,
    HV_X64_MSR_GUEST_OS_ID,
];

#[derive(Default, Copy, Clone)]
pub struct VpFeatures {
    pub proc_features: hv_partition_processor_features,
    pub xsave_features: hv_partition_processor_xsave_features,
    pub synthetic_features: hv_partition_synthetic_processor_features,
}

/// Return the MSR indexes based on supported CPU features
pub fn get_partition_supported_msrs(features: &VpFeatures) -> Vec<u32> {
    let mut msrs: Vec<u32> = Vec::new();
    msrs.extend_from_slice(MSRS_COMMON);

    // SAFETY: access union fields
    unsafe {
        if features.proc_features.__bindgen_anon_1.cet_ss_support() == 1u64 {
            msrs.extend_from_slice(MSRS_CET_SS);
        }
        if features.proc_features.__bindgen_anon_1.rdtscp_support() == 1u64 {
            msrs.push(IA32_MSR_TSC_AUX);
        }
        if features
            .xsave_features
            .__bindgen_anon_1
            .xsave_supervisor_support()
            == 1u64
        {
            msrs.push(MSR_IA32_REGISTER_U_XSS);
        }
        if features
            .synthetic_features
            .__bindgen_anon_1
            .access_synic_regs()
            == 1u64
        {
            msrs.extend_from_slice(MSRS_SYNIC);
        }
        if features
            .synthetic_features
            .__bindgen_anon_1
            .access_partition_reference_tsc()
            == 1u64
        {
            msrs.push(HV_X64_MSR_REFERENCE_TSC);
        }
        if features
            .synthetic_features
            .__bindgen_anon_1
            .access_hypercall_regs()
            == 1u64
        {
            msrs.push(HV_X64_MSR_GUEST_OS_ID);
        }
    }

    /* return all the MSRs we currently support */
    msrs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::slice::from_raw_parts_mut;

    #[test]
    fn test_all_vp_state_components_copy_to_buffer() {
        let mut states: AllVpStateComponents = AllVpStateComponents::default();
        let mut buffer = Buffer::new(HV_PAGE_SIZE, HV_PAGE_SIZE).unwrap();

        for i in 0..VP_STATE_COMPONENTS_BUFFER_SIZE {
            states.buffer[i] = 0xB9;
        }

        //test copy to buffer
        for i in 0..MSHV_VP_STATE_COUNT {
            let len = VP_STATE_COMP_SIZES[i as usize];
            let start = get_vp_state_comp_start_offset(i as usize);
            let end = start + len;
            states.copy_to_or_from_buffer(i as usize, &mut buffer, true);
            // SAFETY: We read less than or equal to buffer length and the slice is valid.
            let buf_arr = unsafe { std::slice::from_raw_parts(buffer.buf, len) };
            assert!(states.buffer[start..end]
                .iter()
                .zip(buf_arr)
                .all(|(a, b)| a == b));
        }
    }

    #[test]
    fn test_all_vp_state_components_copy_from_buffer() {
        let mut states: AllVpStateComponents = AllVpStateComponents::default();
        let buffer = Buffer::new(HV_PAGE_SIZE, HV_PAGE_SIZE).unwrap();
        let mut copy_buffer = Buffer::new(HV_PAGE_SIZE, HV_PAGE_SIZE).unwrap();

        // SAFETY: Safe because the entire buffer is accessible as bytes,
        // modifying them in the form of a byte slice is valid
        let mut_buf = unsafe { from_raw_parts_mut(buffer.buf, buffer.layout.size()) };
        for itm in mut_buf.iter_mut().take(HV_PAGE_SIZE) {
            *itm = 0xA5;
        }

        // SAFETY: buffer is large enough to hold state data
        unsafe { ptr::copy(mut_buf.as_mut_ptr(), copy_buffer.buf, HV_PAGE_SIZE) };

        //test copy to buffer
        for i in 0..MSHV_VP_STATE_COUNT {
            let len = VP_STATE_COMP_SIZES[i as usize];
            let start = get_vp_state_comp_start_offset(i as usize);
            let end = start + len;

            states.copy_to_or_from_buffer(i as usize, &mut copy_buffer, false);
            let buf_arr = &mut_buf[0..len];
            assert!(states.buffer[start..end]
                .iter()
                .zip(buf_arr)
                .all(|(a, b)| a == b));
        }
    }
}
