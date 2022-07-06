use gst::prelude::GstBinExt;
use gst::prelude::ElementExt;

fn pipeline_create(uri: &str) -> Result<gst::Pipeline, glib::error::Error> {
    let pipeline = gst::Pipeline::new(None);
    
    let bin = match gst::parse_bin_from_description(&format!("rtspsrc location={uri} ! decodebin ! appsink"), false) {
        Ok(bin) => bin,
        Err(error) => {
            return Err(error);
        }
    };
    
    pipeline.add(&bin);

    Ok(pipeline)
}

fn main() {
    gst::init();
   
    let pipeline = pipeline_create("rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mp4");

    /*
    pipeline.set_state(gst::State::Playing); 
    loop {

    }
    */

}



