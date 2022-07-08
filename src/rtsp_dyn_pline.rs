use futures::Stream;
use glib::WeakRef;
use gst::Pipeline;
use gst::element_error;
use gst::element_warning;
use gst::prelude::*;

use std::env;

use anyhow::Error;
use derive_more::{Display, Error};
use gst_app::AppSink;

#[derive(Debug, Display, Error)]
#[display(fmt = "Missing element {}", _0)]
struct MissingElement(#[error(not(source))] &'static str);

pub struct RTSPPipeline {
    pub pipeline: Pipeline,
    pub appsinks: std::sync::Arc<std::sync::Mutex<Vec<gst::Element>>>,
}

impl RTSPPipeline {
    pub fn new (uri: &str) -> Result<RTSPPipeline, Error>{
        let mut rtsp_pipeline = RTSPPipeline {
            pipeline: gst::Pipeline::new(None),
            appsinks: std::sync::Arc::new(std::sync::Mutex::new(Vec::new()))
        };
        
        let uridecodebin = gst::ElementFactory::make("uridecodebin", None).map_err(|_| MissingElement("uridecodebin"))?;
        uridecodebin.set_property("uri", uri);
        
        rtsp_pipeline.pipeline.add(&uridecodebin)?;
        let pipeline_weak = rtsp_pipeline.pipeline.downgrade();
        let r_appsinks = std::sync::Arc::clone(&rtsp_pipeline.appsinks);

        uridecodebin.connect_pad_added(move |uridecodebin, src_pad| {
            let pipeline = match pipeline_weak.upgrade() {
                Some(pipeline) => pipeline,
                None => return,
            };

            let appsink = gst::ElementFactory::make("appsink", None).map_err(|_| MissingElement("appsink")).unwrap(); 
            pipeline.add(&appsink);
            uridecodebin.link(&appsink);
           
            let mut appsinks = r_appsinks.lock().unwrap();
            appsinks.push(appsink);
        
        });
        
        Ok(rtsp_pipeline)
    }
}
