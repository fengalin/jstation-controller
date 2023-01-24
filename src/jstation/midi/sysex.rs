use nom::{
    bytes::complete::{tag, take},
    error::{self, Error},
    IResult,
};

use crate::jstation::{
    procedure::{self, Procedure},
    split_bytes,
};
use crate::midi;

const MANUFACTURER_ID: [u8; 3] = [0x0, 0x0, 0x10];
const CHECKSUM_INIT: u8 = MANUFACTURER_ID[0] ^ MANUFACTURER_ID[1] ^ MANUFACTURER_ID[2];
const PRODUCT_ID: u8 = 0x54;

pub struct BufferBuilder {
    buf: Vec<u8>,
    checksum: u8,
}

fn write_checksum(checksum: &mut u8) -> impl FnMut(&u8) + '_ {
    |byte| *checksum ^= *byte
}

impl BufferBuilder {
    pub fn new(chan: midi::Channel, proc_id: u8, version: u8) -> Self {
        let mut buf = vec![
            midi::sysex::TAG,
            MANUFACTURER_ID[0],
            MANUFACTURER_ID[1],
            MANUFACTURER_ID[2],
        ];

        let mut checksum = CHECKSUM_INIT;
        buf.extend(
            [chan.into(), PRODUCT_ID, proc_id]
                .into_iter()
                .chain(split_bytes::from_u8(version))
                .inspect(write_checksum(&mut checksum)),
        );

        BufferBuilder { buf, checksum }
    }

    pub fn push_fixed_size_data(&mut self, data: impl Iterator<Item = u8>) {
        self.buf.extend(
            data.flat_map(split_bytes::from_u8)
                .inspect(write_checksum(&mut self.checksum)),
        );
    }

    #[track_caller]
    pub fn push_variable_size_data(&mut self, data: impl ExactSizeIterator<Item = u8>) {
        let len: u16 = data
            .len()
            .try_into()
            .expect("variable size data length overflow");

        self.buf.extend(
            split_bytes::from_u16(len)
                .into_iter()
                .chain(data.flat_map(split_bytes::from_u8))
                .inspect(write_checksum(&mut self.checksum)),
        );
    }

    pub fn build(mut self) -> Vec<u8> {
        self.buf.extend([self.checksum, midi::sysex::END_TAG]);

        // Set to true to dump buffers
        if false {
            println!(
                "Out Buffer {:?}\n",
                self.buf
                    .iter()
                    .map(|byte| format!("x{byte:02x}"))
                    .collect::<Vec<String>>(),
            );
        }

        self.buf
    }
}

#[derive(Debug)]
pub struct Message {
    pub chan: midi::Channel,
    pub proc: Procedure,
}

pub fn parse(input: &[u8]) -> IResult<&[u8], Message> {
    let (i, _) = tag([
        midi::sysex::TAG,
        MANUFACTURER_ID[0],
        MANUFACTURER_ID[1],
        MANUFACTURER_ID[2],
    ])(input)?;

    let (i, chan) = take(1usize)(i)?;
    let (i, _) = tag([PRODUCT_ID])(i)?;

    let mut checksum = CHECKSUM_INIT ^ chan[0] ^ PRODUCT_ID;

    let (i, proc) = procedure::parse(i, &mut checksum)?;

    let (i, msg_checksum) = take(1usize)(i)?;
    if msg_checksum[0] != checksum {
        log::error!(
            "Expected checksum 0x{checksum:02x}, found 0x{:02x}: {proc:?}",
            msg_checksum[0],
        );
        return Err(nom::Err::Failure(Error::new(i, error::ErrorKind::Verify)));
    }

    let (i, _) = tag([midi::sysex::END_TAG])(i)?;

    Ok((
        i,
        Message {
            chan: midi::Channel::from(chan[0]),
            proc,
        },
    ))
}

pub fn take_u8<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], u8> {
    let (i, bytes) = take(1usize)(i)?;
    let byte = bytes[0];
    *checksum ^= byte;

    Ok((i, byte))
}

pub fn take_split_bytes_bool<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], bool> {
    let (i, bytes) = take(2usize)(i)?;
    *checksum = *checksum ^ bytes[0] ^ bytes[1];

    Ok((i, split_bytes::to_bool(bytes)))
}

#[track_caller]
pub fn take_split_bytes_u8<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], u8> {
    let (i, bytes) = take(2usize)(i)?;
    *checksum = *checksum ^ bytes[0] ^ bytes[1];

    Ok((i, split_bytes::to_u8(bytes)))
}

pub fn take_split_bytes_chan<'i>(
    i: &'i [u8],
    checksum: &mut u8,
) -> IResult<&'i [u8], midi::Channel> {
    let (i, chan) = take_split_bytes_u8(i, checksum)?;

    if chan > 0x0f {
        return Ok((i, midi::Channel::ALL));
    }

    Ok((i, chan.into()))
}

#[track_caller]
pub fn take_split_bytes_u16<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], u16> {
    let (i, bytes) = take(4usize)(i)?;
    *checksum = *checksum ^ bytes[0] ^ bytes[1] ^ bytes[2] ^ bytes[3];

    Ok((i, split_bytes::to_u16(bytes)))
}

#[track_caller]
pub fn take_split_bytes_len<'i>(
    i: &'i [u8],
    checksum: &mut u8,
    expected: u16,
) -> IResult<&'i [u8], ()> {
    let (i, len) = take_split_bytes_u16(i, checksum)?;
    if len != expected {
        log::error!("Expected length {expected}, found {len}");
        return Err(nom::Err::Failure(Error::new(
            i,
            error::ErrorKind::LengthValue,
        )));
    }

    Ok((i, ()))
}
