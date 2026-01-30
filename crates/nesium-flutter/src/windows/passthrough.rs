use std::path::PathBuf;

const PASSTHROUGH_SLANG: &str = r#"#version 450

#pragma stage vertex
layout(location = 0) in vec4 Position;
layout(location = 1) in vec2 TexCoord;
layout(location = 0) out vec2 vTexCoord;

layout(set = 0, binding = 0, std140) uniform UBO {
    mat4 MVP;
} global;

void main() {
    gl_Position = global.MVP * Position;
    vTexCoord = TexCoord;
}

#pragma stage fragment
layout(location = 0) in vec2 vTexCoord;
layout(location = 0) out vec4 FragColor;
layout(set = 0, binding = 2) uniform sampler2D Source;

void main() {
    FragColor = texture(Source, vTexCoord);
}
"#;

pub(crate) fn get_passthrough_preset() -> PathBuf {
    let temp = std::env::temp_dir();
    let slangp = temp.join("nesium_passthrough.slangp");
    let slang = temp.join("passthrough.slang");

    // Use a passthrough shader to copy/scale the texture when no user shader is active.
    let _ = std::fs::write(&slang, PASSTHROUGH_SLANG);

    // Write the preset pointing to the shader file with a sanitized path
    let slang_path = slang.to_string_lossy().replace("\\", "/");
    let _ = std::fs::write(
        &slangp,
        format!("shaders = 1\nshader0 = \"{}\"\n", slang_path),
    );
    slangp
}
