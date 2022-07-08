// authors: mihail8531, danishmakbari

use derive_more::{Display, Error};
use std::{env, fmt::Debug};

#[derive(Debug, Display, Error)]
#[display(fmt = "Missing element {}", _0)]
struct MissingElement(#[error(not(source))] &'static str);


mod rtsp_dyn_pline;

use rtsp_dyn_pline::RTSPPipeline;

fn main() {
    let args: Vec<_> = env::args().collect();
    let uri: &str = if args.len() == 2 {
        args[1].as_ref()
    } else {
        println!("Usage: decodebin file_path");
        std::process::exit(-1)
    };

    gst::init();

    let mut rtsppipeline = match RTSPPipeline::new(uri) {
        Ok(r) => r,
        Err(e) => {eprintln!("Error! {}", e); return},
    };

    loop {
        println!("{:?}", rtsppipeline.get_audio_stream());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    

}
