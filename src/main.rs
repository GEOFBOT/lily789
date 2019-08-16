use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str;

use byteorder::ByteOrder;
use byteorder::LittleEndian;

use nom::bytes::complete::take;
use nom::combinator::map;
use nom::multi::many1;
use nom::number::complete::le_u16;
use nom::IResult;

#[derive(Debug)]
struct RawRecord<'a> {
    tag: u16,
    len: u16,
    dat: &'a [u8],
}

#[derive(Debug)]
enum CalcMode {
    Manual,
    Automatic,
}

#[derive(Debug)]
enum CalcOrder {
    Natural,
    ByColumn,
    ByRow,
}

#[derive(Debug)]
enum SplitType {
    NotSplit,
    VerticalSplit,
    HorizontalSplit,
}

#[derive(Debug)]
enum Record<'a> {
    BeginningOfFile {
        version: u16,
    },
    EndOfFile,
    CalcMode(CalcMode),
    CalcOrder(CalcOrder),
    SplitType(SplitType),
    SplitSync(bool),
    ActiveRange {
        start_col: u16,
        start_row: u16,
        end_col: u16,
        end_row: u16,
    },
    Window1,
    Window1ColWidth,
    Window2,
    Window2ColWidth,
    NamedRange,
    BlankCell {
        format: u8,
        col: u16,
        row: u16,
    },
    IntegerCell {
        format: u8,
        col: u16,
        row: u16,
        val: u16,
    },
    FloatCell {
        format: u8,
        col: u16,
        row: u16,
        val: f64,
    },
    LabelCell {
        format: u8,
        col: u16,
        row: u16,
        val: &'a str,
    },
    FormulaCell,
    DataRange,
    QueryRange,
    PrintRange,
    SortRange,
    FillRange,
    SortKeyRange1,
    DistRange,
    SortKeyRange2,
    GlobalProtect,
    PrintFooter,
    PrintHeader,
    PrintSetup,
    PrintMargins,
    LabelAlignment,
    PrintBorders,
    CurrentGraph,
    NamedGraph,
    CalcCount,
    FormattedPrint,
    CursorLocation,
    SymphonyWindow,
    FormulaValue,
    Password,
    Locked,
    SymphonyQuery,
    SymphonyQueryName,
    SymphonyPrint,
    SymphonyPrintName,
    SymphonyGraph,
    SymphonyGraphName,
    Zoom,
    NumSplitWindows,
    NumScreenRows,
    NumScreenCols,
    NamedRulerRange,
    NamedSheetRange,
    AutoloadCode,
    AutoexecuteMacro,
    QueryParse,
    Unknown(u16),
}

fn parse_record(input: &[u8]) -> IResult<&[u8], RawRecord> {
    let (input, tag) = le_u16(input)?;
    let (input, len) = le_u16(input)?;
    let (input, dat) = take(len)(input)?;
    Ok((input, RawRecord { tag, len, dat }))
}

fn get_record(raw: RawRecord) -> Record {
    match raw.tag {
        0x00 => Record::BeginningOfFile {
            version: LittleEndian::read_u16(raw.dat),
        },
        0x01 => Record::EndOfFile,
        0x02 => Record::CalcMode(match raw.dat[0] {
            0x00 => CalcMode::Manual,
            // TODO investigate 0x01
            // as perhaps a quirk of Quattro Pro?
            0x01 | 0xFF => CalcMode::Automatic,
            _ => panic!(),
        }),
        0x03 => Record::CalcOrder(match raw.dat[0] {
            0x00 => CalcOrder::Natural,
            0x01 => CalcOrder::ByColumn,
            0xFF => CalcOrder::ByRow,
            _ => panic!(),
        }),
        0x04 => Record::SplitType(match raw.dat[0] {
            0x00 => SplitType::NotSplit,
            0x01 => SplitType::VerticalSplit,
            0xFF => SplitType::HorizontalSplit,
            _ => panic!(),
        }),
        0x05 => Record::SplitSync(raw.dat[0] == 0xFF),
        0x06 => Record::ActiveRange {
            start_col: LittleEndian::read_u16(&raw.dat[0..2]),
            start_row: LittleEndian::read_u16(&raw.dat[2..4]),
            end_col: LittleEndian::read_u16(&raw.dat[4..6]),
            end_row: LittleEndian::read_u16(&raw.dat[6..8]),
        },
        0x07 => Record::Window1,
        0x08 => Record::Window1ColWidth,
        0x09 => Record::Window2,
        0x0A => Record::Window2ColWidth,
        0x0B => Record::NamedRange,
        0x0C => Record::BlankCell {
            format: raw.dat[0],
            col: LittleEndian::read_u16(&raw.dat[1..3]),
            row: LittleEndian::read_u16(&raw.dat[3..5]),
        },
        0x0D => Record::IntegerCell {
            format: raw.dat[0],
            col: LittleEndian::read_u16(&raw.dat[1..3]),
            row: LittleEndian::read_u16(&raw.dat[3..5]),
            val: LittleEndian::read_u16(&raw.dat[5..7]),
        },
        0x0E => Record::FloatCell {
            format: raw.dat[0],
            col: LittleEndian::read_u16(&raw.dat[1..3]),
            row: LittleEndian::read_u16(&raw.dat[3..5]),
            val: LittleEndian::read_f64(&raw.dat[5..13]),
        },
        0x0F => Record::LabelCell {
            format: raw.dat[0],
            col: LittleEndian::read_u16(&raw.dat[1..3]),
            row: LittleEndian::read_u16(&raw.dat[3..5]),
            val: str::from_utf8(&raw.dat[5..]).unwrap(),
        },
        0x10 => Record::FormulaCell,
        0x18 => Record::DataRange,
        0x19 => Record::QueryRange,
        0x1A => Record::PrintRange,
        0x1B => Record::SortRange,
        0x1C => Record::FillRange,
        0x1D => Record::SortKeyRange1,
        0x20 => Record::DistRange,
        0x23 => Record::SortKeyRange2,
        0x24 => Record::GlobalProtect,
        0x25 => Record::PrintFooter,
        0x26 => Record::PrintHeader,
        0x27 => Record::PrintSetup,
        0x28 => Record::PrintMargins,
        _ => Record::Unknown(raw.tag),
    }
}

fn main() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/data/KSBASE1.WK1");
    let mut f = File::open(p.to_str().unwrap()).unwrap();
    let mut v: Vec<u8> = vec![];
    f.read_to_end(&mut v).unwrap();
    println!("{:?}", many1(map(parse_record, get_record))(&v));

    println!("Hello, world!");
}
