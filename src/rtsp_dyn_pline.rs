use gst::Pipeline;
use gst::element_warning;
use gst::prelude::*;


use anyhow::Error;
use derive_more::{Display, Error};
use gst_app::AppSink;
use gst_app::app_sink::AppSinkStream;
use gst::MessageView;

#[derive(Debug, Display, Error)]
#[display(fmt = "Missing element {}", _0)]
struct MissingElement(#[error(not(source))] &'static str);

#[derive(Debug, Display, Error)]
#[display(fmt = "Received error from {}: {} (debug: {:?})", src, error, debug)]

struct ErrorMessage {
    src: String,
    error: String,
    debug: Option<String>,
    source: glib::Error,
}

fn print_if_err(res: Result<(), glib::error::BoolError>) -> () {
    match res {
        Ok(_) => (),
        Err(e) => glib::g_printerr!("{}", e),
    }
}

pub struct RTSPPipeline {
    pub pipeline: Pipeline,
    audio_stream: Option<AppSinkStream>,
    video_stream: Option<AppSinkStream>,
}

impl RTSPPipeline {
    pub fn get_audio_stream(&mut self) -> Option<&AppSinkStream> {
        if self.audio_stream.is_some(){
            return Some(&self.audio_stream.as_ref().unwrap());
        }
        match self.pipeline.child_by_name("appsink_audio") {
            None => None,
            Some(sink) => {
                self.audio_stream = Some(sink.dynamic_cast::<AppSink>().unwrap().stream());
                return Some(&self.audio_stream.as_ref().unwrap());
            },
        }
    }

    pub fn get_video_stream(&mut self) -> Option<&AppSinkStream> {
        if self.video_stream.is_some(){
            return Some(&self.video_stream.as_ref().unwrap());
        }
        match self.pipeline.child_by_name("appsink_video") {
            None => None,
            Some(sink) => {
                self.video_stream = Some(sink.dynamic_cast::<AppSink>().unwrap().stream());
                return Some(&self.video_stream.as_ref().unwrap());
            },
        }
    }

    pub fn new (uri: &str) -> Result<RTSPPipeline, Error>{
        let rtsp_pipeline = RTSPPipeline {
            pipeline: gst::Pipeline::new(None),
            audio_stream: None,
            video_stream: None,
        };
        let uridecodebin = gst::ElementFactory::make("uridecodebin", None).map_err(|_| MissingElement("uridecodebin"))?;
        uridecodebin.set_property("uri", uri);
        
        rtsp_pipeline.pipeline.add(&uridecodebin)?;
        let pipeline_weak = rtsp_pipeline.pipeline.downgrade();

        uridecodebin.connect_pad_added(move |uridecodebin, src_pad| {
            let pipeline = match pipeline_weak.upgrade() {
                Some(pipeline) => pipeline,
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
                            uridecodebin,
                            gst::CoreError::Negotiation,
                            ("Failed to get media type from pad {}", src_pad.name())
                        );
    
                        return;
                    }
                    Some(media_type) => media_type,
                }
            };
            if is_audio {
                let appsink = gst::ElementFactory::make("appsink", Some("appsink_audio")).map_err(|_| MissingElement("appsink")).unwrap(); 
                print_if_err(pipeline.add(&appsink));
                
                print_if_err(uridecodebin.link(&appsink));
                
                print_if_err(appsink.sync_state_with_parent());
            }
            if is_video {
                let appsink = gst::ElementFactory::make("appsink", Some("appsink_video")).map_err(|_| MissingElement("appsink")).unwrap(); 
                print_if_err(pipeline.add(&appsink));
                print_if_err(uridecodebin.link(&appsink));
                print_if_err(appsink.sync_state_with_parent());
            }
            print_if_err(uridecodebin.sync_state_with_parent());
        });
        
        rtsp_pipeline.pipeline.set_state(gst::State::Playing)?;

        let bus = rtsp_pipeline.pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        let pipeline_weak = rtsp_pipeline.pipeline.downgrade();
        bus.add_watch(move |_, msg| {
            match msg.view() {
                MessageView::Eos(..) => glib::Continue(false),
                MessageView::Error(err) => {
                    let pipeline = match pipeline_weak.upgrade() {
                        Some(pipeline) => pipeline,
                        None => unimplemented!(),
                    };
                    match pipeline.set_state(gst::State::Null) {
                        Ok(_) => (),
                        Err(e) => glib::g_printerr!("{}", e),
                    }
                    {
                        eprint!("{}", ErrorMessage {
                            src: msg
                                .src()
                                .map(|s| String::from(s.path_string()))
                                .unwrap_or_else(|| String::from("None")),
                            error: err.error().to_string(),
                            debug: err.debug(),
                            source: err.error(),
                        });
                    glib::Continue(false)
                    }
                }
                MessageView::StateChanged(s) => {
                    println!(
                        "State changed from {:?}: {:?} -> {:?} ({:?})",
                        s.src().map(|s| s.path_string()),
                        s.old(),
                        s.current(),
                        s.pending()
                    );
                    glib::Continue(true)
                }
                _ => glib::Continue(true),
            }
        })?;
        Ok(rtsp_pipeline)
    }
}
