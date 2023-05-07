
use std::io::{Cursor, Read, Seek};
use std::thread;
use std::time::Duration;

use bytes::Buf;

use lofty::{AudioFile, ParseOptions};
use lofty::iff::wav;
use reqwest::header::{HeaderValue, RANGE};
use songbird::Call;
use songbird::input::{Container, Input, Metadata, Reader};
use songbird::input::reader::StreamFromURL;
use symphonia_core::io::{ReadOnlySource};
use tokio::runtime::Builder;
use tokio::sync::MutexGuard;
use tokio::time;
use crate::worker::sources::helpers::lofty_wac_codec_to_songbird_codec;

/// Basic URL Player that downloads files from URLs into memory and plays them
/// TODO: Optimize by only loading chunks into memory at a time by chunking downloads
/// TODO: This may require some lower level work inside of Songbird/Finch
/// TODO: This currently only supports .WAV files add support for .OGG, .MP3, .FLAC, and .AIFF
pub async fn url_source(url: &str) -> Input {
    let chunk_size = 500000; // Chunk = 500KB
    let range = HeaderValue::from_str(&format!("bytes={}-{}", 0, &chunk_size)).expect("string provided by format!");
    println!("RANGE: {:?}",range);
    let client = reqwest::Client::new();
    let resp = client.get(url).header(RANGE, range).send().await.unwrap();
    let mut pre : Vec<u8> = vec![];

    let bytes = resp.bytes().await.unwrap().clone();
    let metadata_bytes = bytes.clone(); // This is required because for some reason read_to_end breaks the pre-buf symph

    metadata_bytes.reader().read_to_end(&mut pre).unwrap();
    let mut mock_file : Cursor<Vec<u8>> = Cursor::new(pre);

    let mut mfp = mock_file.clone();
    let parsing_options = ParseOptions::new();
    let tagged_file = wav::WavFile::read_from(&mut mfp, parsing_options).unwrap();
    let properties = tagged_file.properties();

    let x =  Input {
        metadata: Box::new(Metadata {
            track: None,
            artist: None,
            date: None,
            channels: Some(properties.channels()),
            channel: None,
            start_time: None,
            duration: Some(properties.duration()),
            sample_rate: Some(properties.sample_rate()),
            source_url: None,
            title: None,
            thumbnail: None,
        }),
        stereo: properties.channels() >= 2,
        reader: Reader::StreamForURL(StreamFromURL::new(mock_file,url, chunk_size,50000)),
        kind: lofty_wac_codec_to_songbird_codec(tagged_file.properties().format()),
        container: Container::Raw,
        pos: 0,
    };
    return x;
}