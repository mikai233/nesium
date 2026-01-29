use crate::api::video::ShaderParameters;
use crate::frb_generated::StreamSink;
use arc_swap::ArcSwapOption;
use std::sync::Arc;

static SHADER_SINK: ArcSwapOption<StreamSink<ShaderParameters>> = ArcSwapOption::const_empty();

pub fn set_shader_sink(sink: StreamSink<ShaderParameters>) {
    SHADER_SINK.store(Some(Arc::new(sink)));
}

pub fn emit_shader_parameters_update(parameters: ShaderParameters) {
    match &*SHADER_SINK.load() {
        Some(sink) => {
            let _ = sink.add(parameters);
        }
        None => {
            tracing::debug!("Shader parameters update dropped (no sink registered)");
        }
    }
}
