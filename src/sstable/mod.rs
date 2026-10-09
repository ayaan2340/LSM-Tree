pub mod writer;
pub mod reader;

use self::writer::SSTableWriter;

const KEY_LEN_SIZE: usize = 4;
const VALUE_LEN_SIZE: usize = 4;
const OFFSET_SIZE: usize = 8;
const METADATA_SIZE: usize = 1;
const SEQUENCE_NUMBER_SIZE: usize = 8;
const FOOTER_SIZE: usize = 24;
