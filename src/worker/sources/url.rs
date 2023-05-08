use std::io::{Cursor, Read, Seek};
use std::{fmt, thread};
use std::time::Duration;
use bytes::Buf;
use kafka::producer::Producer;
use lofty::{AudioFile, mpeg, ogg, ParseOptions};
use lofty::iff::wav;
use lofty::iff::wav::WavProperties;
use reqwest::header::{HeaderValue, RANGE};
use songbird::Call;
use songbird::input::{Codec, Container, Input, Metadata, Reader};
use songbird::input::codec::OpusDecoderState;
use songbird::input::reader::StreamFromURL;
use symphonia_core::io::{ReadOnlySource};
use tokio::runtime::Builder;
use tokio::sync::MutexGuard;
use tokio::time;
use crate::worker::queue_processor::ErrorReport;
use crate::worker::sources::helpers::{get_extension_from_uri, lofty_wav_codec_to_songbird_codec};
use snafu::{ResultExt, Snafu, Whatever, whatever};


/// Basic URL Player that downloads files from URLs into memory and plays them
pub async fn url_source(url: &str) -> Result<Input, Whatever> {
    // let chunk_size = 250000; // Chunk = 250KB
    let chunk_size = 10000000;
    let range = HeaderValue::from_str(&format!("bytes={}-{}", 0, &chunk_size)).with_whatever_context(|_| format!("Failed to generate range header"))?;
    let client = reqwest::Client::new();
    let resp = client.get(url).header(RANGE, range).send().await.with_whatever_context(|_| format!("Failed to read file from URL"))?;
    let mut pre : Vec<u8> = vec![];

    let bytes = resp.bytes().await.with_whatever_context(|_| format!("Failed to read bytes from file. Malformed?"))?.clone();
    let metadata_bytes = bytes.clone(); // This is required because for some reason read_to_end breaks the pre-buf symph

    metadata_bytes.reader().read_to_end(&mut pre).with_whatever_context(|_| format!("Failed to read bytes into in memory file buffer"))?;
    let mock_file : Cursor<Vec<u8>> = Cursor::new(pre);

    let format = get_extension_from_uri(url);
    let mut metadata: Option<Metadata> = None;
    let mut mfp = mock_file.clone();
    let mut stereo = false;
    let mut codec : Option<Codec> = None;
    let mut container = Container::Raw;
    match format.as_str() {
        //TODO: Ogg is broken see https://github.com/serenity-rs/songbird/issues/85 for fix
        "ogg" => {
            let parsing_options = ParseOptions::new();
            let tagged_file = ogg::OpusFile::read_from(&mut mfp, parsing_options).unwrap();
            let properties = tagged_file.properties();
            metadata = Some(Metadata {
                track: None,
                artist: None,
                date: None,
                channels: Some(properties.channels()),
                channel: None,
                start_time: None,
                duration: Some(properties.duration()),
                sample_rate: Some(properties.input_sample_rate()),
                source_url: None,
                title: None,
                thumbnail: None,
            });
            stereo = properties.channels() >= 2;
            codec = Some(Codec::Opus(OpusDecoderState::new().unwrap()));
            container = Container::Dca {
                first_frame: 0,
            }
        },
        "wav" => {
            let parsing_options = ParseOptions::new();
            let tagged_file = wav::WavFile::read_from(&mut mfp, parsing_options).unwrap();
            let properties = tagged_file.properties();
            metadata = Some(Metadata {
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
            });
            codec = Some(lofty_wav_codec_to_songbird_codec(properties.format()));
            stereo = properties.channels() >= 2
        },
        // "mp3" => {
            // We may support this in the future it is not currently supported because songbird does
            // not support the LAME codec
        // },
        _ => whatever!("Invalid file format. Valid file formats are: .wav and .ogg. using .ogg is recommended as .wav will be downscaled to .ogg internally to support the discord api at the cost of extra memory and cpu cycles.")
    }

    let x =  Input {
        metadata: Box::new(metadata.unwrap()),
        stereo: stereo,
        // reader: Reader::StreamForURL(StreamFromURL::new(mock_file,url, chunk_size,250000)),
        reader: Reader::Extension(Box::new(mock_file)),
        kind: codec.unwrap(),
        container: container,
        pos: 0,
    };
    println!("{:?}",x);
    return Ok(x);
}