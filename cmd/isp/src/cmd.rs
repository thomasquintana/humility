// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::protocol::*;
use anyhow::Result;

fn do_command(
    port: &mut dyn serialport::SerialPort,
    tag: CommandTag,
    command_resp: ResponseCode,
    args: &[u32],
) -> Result<Vec<u32>> {
    send_command(port, tag, args)?;

    read_response(port, command_resp)
}

fn send_data(
    port: &mut dyn serialport::SerialPort,
    data: &[u8],
    code: ResponseCode,
) -> Result<()> {
    crate::protocol::send_data(port, data)?;
    read_response(port, code)?;
    Ok(())
}

fn recv_data(
    port: &mut dyn serialport::SerialPort,
    cnt: u32,
    code: ResponseCode,
) -> Result<Vec<u8>> {
    let r = crate::protocol::recv_data(port, cnt)?;
    read_response(port, code)?;
    Ok(r)
}

pub fn do_save_keystore(port: &mut dyn serialport::SerialPort) -> Result<()> {
    let args = [
        // Arg 0 =  WriteNonVolatile
        KeyProvisionCmds::WriteNonVolatile as u32,
        // Arg 1 = Memory ID (0 = internal flash)
        0_u32,
    ];

    let _ = do_command(
        port,
        CommandTag::KeyProvision,
        ResponseCode::Generic,
        &args,
    )?;

    Ok(())
}

pub fn do_enroll(port: &mut dyn serialport::SerialPort) -> Result<()> {
    let args = [
        // Arg =  Enroll
        KeyProvisionCmds::Enroll as u32,
    ];

    let _ = do_command(
        port,
        CommandTag::KeyProvision,
        ResponseCode::Generic,
        &args,
    )?;

    Ok(())
}

pub fn do_generate_uds(port: &mut dyn serialport::SerialPort) -> Result<()> {
    let args = [
        // Arg 0 =  SetIntrinsicKey
        KeyProvisionCmds::SetIntrinsicKey as u32,
        // Arg 1 = UDS
        KeyType::Uds as u32,
        // Arg 2 = size
        32_u32,
    ];

    let _ = do_command(
        port,
        CommandTag::KeyProvision,
        ResponseCode::Generic,
        &args,
    )?;

    Ok(())
}

pub fn do_isp_write_keystore(
    port: &mut dyn serialport::SerialPort,
    data: &[u8],
) -> Result<()> {
    let args = [KeyProvisionCmds::WriteKeyStore as u32];

    let _ = do_command(
        port,
        CommandTag::KeyProvision,
        ResponseCode::KeyProvision,
        &args,
    )?;

    send_data(port, data, ResponseCode::Generic)
}

pub fn do_isp_read_memory(
    port: &mut dyn serialport::SerialPort,
    address: u32,
    cnt: u32,
) -> Result<Vec<u8>> {
    let args = [
        // Arg0 = address
        address, // Arg1 = length
        cnt,     // Arg2 = memory type
        0x0,
    ];

    let _ = do_command(
        port,
        CommandTag::ReadMemory,
        ResponseCode::ReadMemory,
        &args,
    )?;

    recv_data(port, cnt, ResponseCode::Generic)
}

/// Variant of `do_isp_read_memory` that reads a compile-time byte count, which
/// means it can return a precisely sized allocation that can be treated as a
/// fixed length array without further work.
pub fn do_isp_read_memory_array<const N: usize>(
    port: &mut dyn serialport::SerialPort,
    address: u32,
) -> Result<Box<[u8; N]>> {
    let cnt = u32::try_from(N).unwrap();
    let bytes = do_isp_read_memory(port, address, cnt)?;
    Ok(crate::fixed_vec(bytes)
        .expect("do_isp_read_memory returned wrong vec length!"))
}

pub fn do_isp_write_memory(
    port: &mut dyn serialport::SerialPort,
    address: u32,
    data: &[u8],
) -> Result<()> {
    let args = [
        // arg 0 = address
        address,
        // arg 1 = len
        data.len() as u32,
        // arg 2 = memory type
        0x0_u32,
    ];

    let _ = do_command(
        port,
        CommandTag::WriteMemory,
        ResponseCode::Generic,
        &args,
    )?;

    send_data(port, data, ResponseCode::Generic)
}

pub fn do_isp_flash_erase_all(
    port: &mut dyn serialport::SerialPort,
) -> Result<()> {
    let args = [
        // Erase internal flash
        0x0_u32,
    ];

    let _ = do_command(
        port,
        CommandTag::FlashEraseAll,
        ResponseCode::Generic,
        &args,
    )?;

    Ok(())
}

pub fn do_isp_get_property(
    port: &mut dyn serialport::SerialPort,
    prop: BootloaderProperty,
) -> Result<Vec<u32>> {
    let args = [
        // Arg 0 = property
        prop as u32,
    ];

    do_command(port, CommandTag::GetProperty, ResponseCode::GetProperty, &args)
}

pub fn do_isp_last_error(
    port: &mut dyn serialport::SerialPort,
) -> Result<Vec<u32>> {
    let args = [
        // Arg 0 = LastCRC
        BootloaderProperty::CRCStatus as u32,
        // Arg 1 = Last error
        1_u32,
    ];

    do_command(port, CommandTag::GetProperty, ResponseCode::GetProperty, &args)
}

pub fn do_ping(port: &mut dyn serialport::SerialPort) -> Result<()> {
    process_ping(port)
}
