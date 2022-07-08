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
    pub appsinks: Vec<AppSink>,
}

impl RTSPPipeline {
    pub fn new (uri: &str) -> Result<RTSPPipeline, Error>{
        let pipeline  = Self::new_pipeline(uri)?;
        let appsinks = Vec::new();
        for el in pipeline.children() {
            println!("{}", el.name());
        }
        Ok(RTSPPipeline { pipeline: pipeline, appsinks: appsinks})
    }

    fn new_pipeline(uri: &str) -> Result<Pipeline, Error> {
        let pipeline = gst::Pipeline::new(None);
        let uridecodebin = gst::ElementFactory::make("uridecodebin", None).map_err(|_| MissingElement("uridecodebin"))?;

        uridecodebin.set_property("uri", uri);

        pipeline.add(&uridecodebin)?;
        let pipeline_weak = pipeline.downgrade();

        let uridecodebin_weak = uridecodebin.downgrade();


        uridecodebin.connect_pad_added(move |dbin, src_pad| {
            let pipeline = match pipeline_weak.upgrade() {
                Some(pipeline) => pipeline,
                None => return,
            };

            let uridecodebin= match uridecodebin_weak.upgrade() {
                Some(uridecodebin) => uridecodebin,
                None => return,
            };

            let (is_audio, is_video) = {
                let media_type = src_pad.current_caps().and_then(|caps| {
                    caps.structure(0).map(|s| {
                        let name = s.name();
                        (name.starts_with("audio/x-raw"), name.starts_with("video/x-raw"))
                    })
                });

                match media_type {
                    None => {
                        element_warning!(
                            dbin,
                            gst::CoreError::Negotiation,
                            ("Failed to get media type from pad {}", src_pad.name())
                        );

                        return;
                    }
                    Some(media_type) => media_type,
                }
            };

            let insert_sink = || -> Result<(), Error> {
                let appsink = gst::ElementFactory::make("appsink", None)
                    .map_err(|_| MissingElement("appsink"))?; 
                pipeline.add(&appsink)?;
                uridecodebin.link(&appsink)?;
                Ok(())
            };

            if let Err(err) = insert_sink() {
                #[cfg(not(feature = "v1_10"))]
                element_error!(
                    dbin,
                    gst::LibraryError::Failed,
                    ("Failed to insert sink"),
                    ["{}", err]
                );
            }
        });
        // let a = Box::new(vec![1]);
        // pipeline.connect_child_added(|pipeline, obj, s| {
        //     println!("{}", a[0]);
        // }
        // );
        // println!("{}", a[0]);

        Ok(pipeline)
    }
}
